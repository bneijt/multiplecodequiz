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

    /// Path to the SQLite database file (created if it doesn't exist)
    #[arg(long, default_value = "embeddings.db")]
    db: PathBuf,

    /// Output path for quiz_data.json
    #[arg(long, default_value = "quiz_data.json")]
    output: PathBuf,

    /// Minimum number of statements in a function body to include
    #[arg(long, default_value_t = 3)]
    min_stmts: usize,

    /// Maximum number of statements in a function body to include
    #[arg(long, default_value_t = 80)]
    max_stmts: usize,

    /// Number of diverse chunks to select for the quiz
    #[arg(long, default_value_t = 20)]
    target: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("=== Step 1: Parsing .rs files in {:?} ===", args.repo);
    let chunks = parser::extract_chunks_from_repo(&args.repo, args.min_stmts, args.max_stmts)?;
    if chunks.is_empty() {
        anyhow::bail!("No suitable code chunks found in the repository.");
    }

    println!(
        "\n=== Step 2: Embedding chunks and storing in {:?} ===",
        args.db
    );
    let conn = embedder::open_db(args.db.to_str().unwrap())?;
    embedder::embed_and_store(&conn, &chunks).await?;

    println!(
        "\n=== Step 3: Selecting {} most diverse chunks ===",
        args.target
    );
    diversity::select_diverse(&conn, args.target)?;

    println!("\n=== Step 4: Generating descriptions with phi4-mini:3.8b ===");
    describer::describe_selected_chunks(&conn).await?;

    println!("\n=== Step 5: Building quiz entries with distractors ===");
    let entries = distractors::build_quiz_entries(&conn)?;

    println!("\n=== Step 6: Exporting to {:?} ===", args.output);
    export::export_json(&entries, &args.output)?;

    println!("\nDone! Run 'trunk serve' in crates/frontend to start the quiz.");
    Ok(())
}
