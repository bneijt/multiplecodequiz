use anyhow::Result;
use polarisdb::prelude::*;
use polarisdb::AsyncCollection;
use rig::embeddings::EmbeddingModel as _;
use rig::providers::ollama;

use crate::parser::CodeChunk;

/// Number of dimensions produced by nomic-embed-text.
pub const EMBEDDING_DIMS: usize = 768;

/// Open or create the PolarisDB collection at `db_path`.
pub async fn open_db(db_path: &str) -> Result<AsyncCollection> {
    let config = CollectionConfig::new(EMBEDDING_DIMS, DistanceMetric::Cosine);
    let collection = AsyncCollection::open_or_create(db_path.to_string(), config).await?;
    Ok(collection)
}

/// Consume a lazy iterator of `CodeChunk`s, embedding and inserting each one
/// into the PolarisDB collection as it arrives. This keeps memory usage flat
/// even for very large codebases.
///
/// Metadata stored in the `Payload`:
///   - `file_path`, `fn_name`, `body`  — source info
///   - `selected`  — "0" until diversity selection marks it "1"
///   - `description` — empty until the LLM fills it in
pub async fn embed_and_store(
    collection: &AsyncCollection,
    chunks: impl Iterator<Item = Result<CodeChunk>>,
) -> Result<usize> {
    let client = ollama::Client::new();
    let model = client.embedding_model(ollama::NOMIC_EMBED_TEXT);

    let mut count = 0usize;
    for chunk_result in chunks {
        let chunk = chunk_result?;

        if chunk.body.len() > 1000 {
            continue;
        }

        // Embed this single chunk
        let embeddings = model.embed_texts(vec![chunk.body.clone()]).await?;
        let vec_f32: Vec<f32> = embeddings[0].vec.iter().map(|&v| v as f32).collect();

        let payload = Payload::new()
            .with_field("file_path", chunk.file_path.clone())
            .with_field("fn_name", chunk.fn_name.clone())
            .with_field("body", chunk.body.clone())
            .with_field("selected", "0")
            .with_field("description", "");

        // Skip insert if an identical vector already exists in the DB.
        // This makes re-runs idempotent without needing a separate dedup key.
        let nearest = collection.search(&vec_f32, 1, None).await;
        if nearest.first().map_or(false, |r| r.distance < 1e-6) {
            continue;
        }

        collection.insert_auto(vec_f32, payload).await?;
        count += 1;

        if count % 10 == 0 {
            println!("  Embedded {} chunks...", count);
        }
    }

    collection.flush().await?;
    Ok(count)
}
