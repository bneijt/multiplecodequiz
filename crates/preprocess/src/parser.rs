use anyhow::{Context, Result};
use quote::quote;
use std::path::Path;
use syn::{visit::Visit, ImplItemFn, ItemFn};
use walkdir::WalkDir;

/// Format a syn::Block using prettyplease.
/// Wraps the block in a dummy function, formats the whole file, then
/// extracts just the body back out.
/// Returns `None` if the block cannot be formatted (e.g. contains unstable
/// `macro foo() {}` syntax nested inside expressions that prettyplease does
/// not support — even when deeply nested inside for loops etc).
fn fmt_block(block: &syn::Block) -> Option<String> {
    let wrapper = quote! { fn __wrapper() #block };
    let Ok(ast) = syn::parse2::<syn::File>(wrapper) else {
        return None;
    };
    // prettyplease panics on Item::Verbatim (e.g. `macro foo() {}` syntax).
    // catch_unwind lets us treat those blocks as unformattable and skip them.
    let result = std::panic::catch_unwind(|| prettyplease::unparse(&ast));
    let formatted = result.ok()?;
    // Strip the "fn __wrapper() {\n" prefix and trailing "}\n" suffix
    let start = formatted.find('{').map(|i| i + 1).unwrap_or(0);
    let end = formatted.rfind('}').unwrap_or(formatted.len());
    let inner = &formatted[start..end];
    Some(dedent(inner).trim().to_string())
}

/// Remove the common leading whitespace from all non-empty lines.
fn dedent(s: &str) -> String {
    let min_indent = s
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    s.lines()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub file_path: String,
    pub fn_name: String,
    pub body: String,
}

struct ChunkCollector {
    chunks: Vec<CodeChunk>,
    file_path: String,
    min_stmts: usize,
    max_stmts: usize,
}

impl ChunkCollector {
    fn new(file_path: String, min_stmts: usize, max_stmts: usize) -> Self {
        Self {
            chunks: Vec::new(),
            file_path,
            min_stmts,
            max_stmts,
        }
    }

    fn accept_block(&self, block: &syn::Block) -> bool {
        let count = block.stmts.len();
        count >= self.min_stmts && count <= self.max_stmts
    }
}

impl<'ast> Visit<'ast> for ChunkCollector {
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if self.accept_block(&node.block) {
            let fn_name = node.sig.ident.to_string();
            if let Some(body) = fmt_block(&node.block) {
                self.chunks.push(CodeChunk {
                    file_path: self.file_path.clone(),
                    fn_name,
                    body,
                });
            }
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if self.accept_block(&node.block) {
            let fn_name = node.sig.ident.to_string();
            if let Some(body) = fmt_block(&node.block) {
                self.chunks.push(CodeChunk {
                    file_path: self.file_path.clone(),
                    fn_name,
                    body,
                });
            }
        }
        syn::visit::visit_item_fn(self, node);
    }
}

/// Returns a lazy iterator of `CodeChunk`s extracted from all `.rs` files under
/// `repo_path`. Files are parsed on demand, so only one file's worth of chunks
/// is held in memory at a time — suitable for huge codebases.
pub fn iter_chunks_from_repo(
    repo_path: &Path,
    min_stmts: usize,
    max_stmts: usize,
) -> impl Iterator<Item = Result<CodeChunk>> {
    WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .flat_map(move |entry| {
            let path = entry.into_path();
            let src = match std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))
            {
                Ok(s) => s,
                Err(e) => return vec![Err(e)],
            };

            let ast = match syn::parse_file(&src) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Warning: failed to parse {}: {}", path.display(), e);
                    return vec![];
                }
            };

            let mut collector =
                ChunkCollector::new(path.display().to_string(), min_stmts, max_stmts);
            collector.visit_file(&ast);
            collector.chunks.into_iter().map(Ok).collect()
        })
}
