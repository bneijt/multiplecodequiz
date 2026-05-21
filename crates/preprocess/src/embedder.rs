use anyhow::Result;
use rig::embeddings::EmbeddingModel as _;
use rig::providers::ollama;
use rusqlite::{Connection, params};

use crate::parser::CodeChunk;

pub fn open_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS chunks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path   TEXT NOT NULL,
            fn_name     TEXT NOT NULL,
            body        TEXT NOT NULL,
            embedding   BLOB,
            selected    INTEGER NOT NULL DEFAULT 0,
            description TEXT
        );
    ")?;
    Ok(conn)
}

pub async fn embed_and_store(conn: &Connection, chunks: &[CodeChunk]) -> Result<()> {
    let client = ollama::Client::new();
    let model = client.embedding_model(ollama::NOMIC_EMBED_TEXT);

    // Process in batches of 32 to avoid overwhelming Ollama
    const BATCH: usize = 32;
    let total = chunks.len();

    for (batch_idx, batch) in chunks.chunks(BATCH).enumerate() {
        let texts: Vec<String> = batch.iter().map(|c| c.body.clone()).collect();
        println!(
            "Embedding batch {}/{} ({} chunks)...",
            batch_idx + 1,
            (total + BATCH - 1) / BATCH,
            texts.len()
        );

        let embeddings: Vec<rig::embeddings::Embedding> = model.embed_texts(texts).await?;

        for (chunk, embedding) in batch.iter().zip(embeddings.iter()) {
            let blob = vec_to_blob(&embedding.vec);
            conn.execute(
                "INSERT INTO chunks (file_path, fn_name, body, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![chunk.file_path, chunk.fn_name, chunk.body, blob],
            )?;
        }
    }

    println!("Stored {} embeddings in database", total);
    Ok(())
}

pub fn vec_to_blob(v: &[f64]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub fn blob_to_vec(b: &[u8]) -> Vec<f64> {
    b.chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}
