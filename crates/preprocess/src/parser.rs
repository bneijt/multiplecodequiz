use anyhow::{Context, Result};
use quote::ToTokens;
use std::path::Path;
use syn::{visit::Visit, ImplItemFn, ItemFn};
use walkdir::WalkDir;

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
            let body = node.block.to_token_stream().to_string();
            self.chunks.push(CodeChunk {
                file_path: self.file_path.clone(),
                fn_name,
                body,
            });
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if self.accept_block(&node.block) {
            let fn_name = node.sig.ident.to_string();
            let body = node.block.to_token_stream().to_string();
            self.chunks.push(CodeChunk {
                file_path: self.file_path.clone(),
                fn_name,
                body,
            });
        }
        syn::visit::visit_item_fn(self, node);
    }
}

pub fn extract_chunks_from_repo(
    repo_path: &Path,
    min_stmts: usize,
    max_stmts: usize,
) -> Result<Vec<CodeChunk>> {
    let mut all_chunks = Vec::new();

    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let path = entry.path();
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let ast = match syn::parse_file(&src) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", path.display(), e);
                continue;
            }
        };

        let mut collector = ChunkCollector::new(
            path.display().to_string(),
            min_stmts,
            max_stmts,
        );
        collector.visit_file(&ast);
        all_chunks.extend(collector.chunks);
    }

    println!("Extracted {} chunks from .rs files", all_chunks.len());
    Ok(all_chunks)
}
