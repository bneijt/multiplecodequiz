use anyhow::Result;
use polarisdb::prelude::*;
use polarisdb::{AsyncCollection, SearchResult};

pub struct QuizEntry {
    pub body: String,
    pub correct: String,
    pub distractors: [String; 3],
}

pub async fn build_quiz_entries(collection: &AsyncCollection) -> Result<Vec<QuizEntry>> {
    // Collect all selected chunks that have a description
    let inner = collection.inner();
    let n = inner.len();

    let mut selected: Vec<(u64, Vec<f32>, Payload)> = Vec::new();
    for id in 1..=(n as u64) {
        if let Some((vec, payload)) = inner.get(id) {
            let is_selected = payload.get_str("selected") == Some("1");
            let has_description = payload
                .get_str("description")
                .map(|d: &str| !d.is_empty())
                .unwrap_or(false);
            if is_selected && has_description {
                selected.push((id, vec, payload));
            }
        }
    }

    println!(
        "Building quiz entries with distractors for {} chunks...",
        selected.len()
    );

    let mut entries: Vec<QuizEntry> = Vec::with_capacity(selected.len());

    for (id, vec, payload) in &selected {
        let body = payload.get_str("body").unwrap_or("").to_string();
        let correct = payload.get_str("description").unwrap_or("").to_string();

        // Use PolarisDB ANN search to find the 4 nearest neighbours.
        // The first result is the vector itself (distance ~0), so we skip it
        // and take the next 3 as distractors.
        let neighbors: Vec<SearchResult> = collection.search(vec, 5, None).await;

        let distractors: Vec<String> = neighbors
            .into_iter()
            .filter(|r| r.id != *id) // exclude self
            .filter_map(|r| {
                r.payload.and_then(|p: Payload| {
                    let desc = p.get_str("description")?.to_string();
                    if desc.is_empty() { None } else { Some(desc) }
                })
            })
            .take(3)
            .collect();

        // Pad with empty strings if there aren't 3 neighbours with descriptions
        let d0 = distractors.get(0).cloned().unwrap_or_default();
        let d1 = distractors.get(1).cloned().unwrap_or_default();
        let d2 = distractors.get(2).cloned().unwrap_or_default();

        entries.push(QuizEntry {
            body,
            correct,
            distractors: [d0, d1, d2],
        });
    }

    Ok(entries)
}
