use anyhow::Result;
use rusqlite::Connection;

use crate::embedder::blob_to_vec;

fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // treat zero vectors as maximally distant
    }
    1.0 - (dot / (norm_a * norm_b))
}

/// Greedy max-min diversity selection:
/// Repeatedly picks the chunk whose minimum distance to the already-selected
/// set is largest (i.e. most dissimilar to all already chosen chunks).
pub fn select_diverse(conn: &Connection, target: usize) -> Result<()> {
    // Load all rows with embeddings
    let mut stmt = conn.prepare("SELECT id, embedding FROM chunks WHERE embedding IS NOT NULL")?;
    let rows: Vec<(i64, Vec<f64>)> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((id, blob))
        })?
        .filter_map(|r| r.ok())
        .map(|(id, blob)| (id, blob_to_vec(&blob)))
        .collect();

    let n = rows.len();
    if n == 0 {
        anyhow::bail!("No chunks with embeddings found in database");
    }

    let actual_target = target.min(n);
    println!(
        "Selecting {} most diverse chunks from {} total...",
        actual_target, n
    );

    // min_dist[i] = min cosine distance from chunk i to any selected chunk
    let mut min_dist = vec![f64::MAX; n];
    let mut selected_indices: Vec<usize> = Vec::with_capacity(actual_target);

    // Seed: pick the first chunk
    selected_indices.push(0);
    for i in 0..n {
        min_dist[i] = cosine_distance(&rows[0].1, &rows[i].1);
    }

    while selected_indices.len() < actual_target {
        // Find the chunk with max min_dist (most dissimilar to all selected)
        let next = (0..n)
            .filter(|i| !selected_indices.contains(i))
            .max_by(|&a, &b| min_dist[a].partial_cmp(&min_dist[b]).unwrap())
            .unwrap();

        selected_indices.push(next);

        // Update min_dist for all remaining chunks
        let new_vec = &rows[next].1;
        for i in 0..n {
            let d = cosine_distance(new_vec, &rows[i].1);
            if d < min_dist[i] {
                min_dist[i] = d;
            }
        }

        if selected_indices.len() % 10 == 0 {
            println!("  Selected {}/{}", selected_indices.len(), actual_target);
        }
    }

    // Mark selected chunks in DB
    let selected_ids: Vec<i64> = selected_indices.iter().map(|&i| rows[i].0).collect();
    for id in &selected_ids {
        conn.execute("UPDATE chunks SET selected = 1 WHERE id = ?1", [id])?;
    }

    println!("Marked {} chunks as selected", selected_ids.len());
    Ok(())
}
