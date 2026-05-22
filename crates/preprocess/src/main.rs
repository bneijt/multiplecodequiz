mod describer;
mod distractors;
mod diversity;
mod embedder;
mod export;
mod parser;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "preprocess")]
#[command(about = "Build a code quiz dataset from a Rust repository")]
struct Args {
    /// Path to the Rust repository to process
    #[arg(long)]
    repo: PathBuf,

    /// Path to the PolarisDB collection directory (created if it doesn't exist)
    #[arg(long, default_value = "embeddings.db")]
    db: PathBuf,

    /// Output path for quiz_data.json
    #[arg(long, default_value = "quiz_data.json")]
    output: PathBuf,

    /// Minimum number of statements in a function body to include
    #[arg(long, default_value_t = 3)]
    min_stmts: usize,

    /// Maximum number of statements in a function body to include
    #[arg(long, default_value_t = 40)]
    max_stmts: usize,

    /// Number of diverse chunks to select for the quiz
    #[arg(long, default_value_t = 20)]
    target: usize,

    /// Number of chunks to embed
    #[arg(long, default_value_t = 1000)]
    embed: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("=== Step 1: Parsing .rs files in {:?} ===", args.repo);
    let chunks = parser::iter_chunks_from_repo(&args.repo, args.min_stmts, args.max_stmts);

    println!(
        "\n=== Step 2: Embedding chunks and storing in {:?} ===",
        args.db
    );
    let collection = embedder::open_db(args.db.to_str().unwrap()).await?;
    let total = embedder::embed_and_store(&collection, chunks.take(args.embed)).await?;
    if total == 0 {
        anyhow::bail!("No suitable code chunks found in the repository.");
    }
    println!("Stored {} embeddings in collection", total);

    println!(
        "\n=== Step 3: Selecting {} most diverse chunks ===",
        args.target
    );
    diversity::select_diverse(&collection, args.target).await?;

    println!("\n=== Step 4: Generating descriptions with phi4-mini:3.8b ===");
    describer::describe_selected_chunks(&collection).await?;

    println!("\n=== Step 5: Building quiz entries with distractors ===");
    let entries = distractors::build_quiz_entries(&collection).await?;

    println!("\n=== Step 6: Exporting to {:?} ===", args.output);
    export::export_json(&entries, &args.output)?;

    println!("\nDone! Run 'trunk serve' in crates/frontend to start the quiz.");
    Ok(())
}
