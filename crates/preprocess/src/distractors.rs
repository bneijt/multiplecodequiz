use anyhow::Result;
use rusqlite::Connection;

use crate::embedder::blob_to_vec;

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

pub struct ChunkWithData {
    pub id: i64,
    pub body: String,
    pub description: String,
    pub embedding: Vec<f64>,
}

pub struct QuizEntry {
    pub body: String,
    pub correct: String,
    pub distractors: [String; 3],
}

pub fn build_quiz_entries(conn: &Connection) -> Result<Vec<QuizEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, body, description, embedding FROM chunks WHERE selected = 1 AND description IS NOT NULL AND embedding IS NOT NULL"
    )?;

    let chunks: Vec<ChunkWithData> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let body: String = row.get(1)?;
            let description: String = row.get(2)?;
            let blob: Vec<u8> = row.get(3)?;
            Ok((id, body, description, blob))
        })?
        .filter_map(|r| r.ok())
        .map(|(id, body, description, blob)| ChunkWithData {
            id,
            body,
            description,
            embedding: blob_to_vec(&blob),
        })
        .collect();

    println!("Building quiz entries with distractors for {} chunks...", chunks.len());

    let entries = chunks
        .iter()
        .map(|chunk| {
            // Find the 3 other chunks with highest cosine similarity to this one
            // (closest in embedding space = most plausible wrong answers)
            let mut similarities: Vec<(usize, f64)> = chunks
                .iter()
                .enumerate()
                .filter(|(_, other)| other.id != chunk.id)
                .map(|(i, other)| {
                    let sim = cosine_similarity(&chunk.embedding, &other.embedding);
                    (i, sim)
                })
                .collect();

            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let distractors: [String; 3] = [
                chunks[similarities[0].0].description.clone(),
                chunks[similarities[1].0].description.clone(),
                chunks[similarities[2].0].description.clone(),
            ];

            QuizEntry {
                body: chunk.body.clone(),
                correct: chunk.description.clone(),
                distractors,
            }
        })
        .collect();

    Ok(entries)
}
