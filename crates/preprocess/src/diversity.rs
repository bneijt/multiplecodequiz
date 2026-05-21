use anyhow::Result;
use polarisdb::prelude::*;
use polarisdb::AsyncCollection;

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // treat zero vectors as maximally distant
    }
    1.0 - (dot / (norm_a * norm_b))
}

/// Greedy max-min diversity selection over all vectors in the collection.
/// Marks `selected = "1"` on the `target` most mutually dissimilar chunks.
pub async fn select_diverse(collection: &AsyncCollection, target: usize) -> Result<()> {
    // Collect all (id, vector, payload) from the collection via the sync inner view.
    // We snapshot into a Vec here; diversity selection is inherently an all-pairs
    // computation so we need everything in memory at once anyway.
    let inner = collection.inner();
    let n = inner.len();
    if n == 0 {
        anyhow::bail!("No chunks found in collection");
    }

    // Gather all entries. PolarisDB auto-assigns sequential u64 IDs starting at 1.
    let mut rows: Vec<(u64, Vec<f32>, Payload)> = Vec::with_capacity(n);
    for id in 1..=(n as u64) {
        if let Some((vec, payload)) = inner.get(id) {
            rows.push((id, vec, payload));
        }
    }

    if rows.is_empty() {
        anyhow::bail!("Could not read any entries from the collection");
    }

    let actual_n = rows.len();
    let actual_target = target.min(actual_n);
    println!(
        "Selecting {} most diverse chunks from {} total...",
        actual_target, actual_n
    );

    // Reset selected flag on all entries
    for (id, vec, payload) in &rows {
        // Rebuild payload with selected = "0" (set() is &mut; use with_field instead)
        let reset_payload = Payload::new()
            .with_field("file_path", payload.get_str("file_path").unwrap_or(""))
            .with_field("fn_name", payload.get_str("fn_name").unwrap_or(""))
            .with_field("body", payload.get_str("body").unwrap_or(""))
            .with_field("selected", "0")
            .with_field(
                "description",
                payload.get_str("description").unwrap_or(""),
            );
        collection.update(*id, vec.clone(), reset_payload).await?;
    }

    // Greedy max-min farthest-point sampling
    let mut min_dist = vec![f32::MAX; actual_n];
    let mut selected_indices: Vec<usize> = Vec::with_capacity(actual_target);

    // Seed with first chunk
    selected_indices.push(0);
    for i in 0..actual_n {
        min_dist[i] = cosine_distance(&rows[0].1, &rows[i].1);
    }

    while selected_indices.len() < actual_target {
        let next = (0..actual_n)
            .filter(|i| !selected_indices.contains(i))
            .max_by(|&a, &b| min_dist[a].partial_cmp(&min_dist[b]).unwrap())
            .unwrap();

        selected_indices.push(next);

        let new_vec = &rows[next].1;
        for i in 0..actual_n {
            let d = cosine_distance(new_vec, &rows[i].1);
            if d < min_dist[i] {
                min_dist[i] = d;
            }
        }

        if selected_indices.len() % 10 == 0 {
            println!("  Selected {}/{}", selected_indices.len(), actual_target);
        }
    }

    // Write selected = "1" back for chosen entries
    for idx in &selected_indices {
        let (id, vec, payload) = &rows[*idx];
        let updated_payload = Payload::new()
            .with_field("file_path", payload.get_str("file_path").unwrap_or(""))
            .with_field("fn_name", payload.get_str("fn_name").unwrap_or(""))
            .with_field("body", payload.get_str("body").unwrap_or(""))
            .with_field("selected", "1")
            .with_field(
                "description",
                payload.get_str("description").unwrap_or(""),
            );
        collection.update(*id, vec.clone(), updated_payload).await?;
    }

    collection.flush().await?;
    println!("Marked {} chunks as selected", selected_indices.len());
    Ok(())
}
