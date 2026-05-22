use anyhow::Result;
use polarisdb::prelude::*;
use polarisdb::AsyncCollection;
use rig::completion::Prompt;
use rig::providers::ollama;

const LLM_MODEL: &str = "phi4-mini:3.8b";

const SYSTEM_PROMPT: &str = "\
You are a Rust code analyst. When given a Rust function body, respond with exactly one \
sentence describing what the code does functionally. Do not mention variable names, \
function names, or implementation details — only the purpose. \
Do not include any preamble, explanation, or punctuation beyond the single sentence.";

pub async fn describe_selected_chunks(collection: &AsyncCollection) -> Result<()> {
    let client = ollama::Client::new();
    let agent = client.agent(LLM_MODEL).preamble(SYSTEM_PROMPT).build();

    // Collect all selected chunks that have no description yet
    let inner = collection.inner();
    let n = inner.len();

    let mut pending: Vec<(u64, Vec<f32>, Payload)> = Vec::new();
    for id in 1..=(n as u64) {
        if let Some((vec, payload)) = inner.get(id) {
            let is_selected = payload.get_str("selected") == Some("1");
            let has_description = payload
                .get_str("description")
                .map(|d: &str| !d.is_empty())
                .unwrap_or(false);
            if is_selected && !has_description {
                pending.push((id, vec, payload));
            }
        }
    }

    let total = pending.len();
    println!("Generating descriptions for {} selected chunks...", total);

    for (idx, (id, vec, payload)) in pending.into_iter().enumerate() {
        let body = payload.get_str("body").unwrap_or("").to_string();
        let prompt = format!(
            "Describe what this Rust code does in one sentence:\n\n{}",
            body
        );

        let description: String = match agent.prompt(prompt.as_str()).await {
            Ok(d) => d.trim().to_string(),
            Err(e) => {
                eprintln!("  Warning: failed to describe chunk {}: {}", id, e);
                continue;
            }
        };

        let updated_payload = Payload::new()
            .with_field("file_path", payload.get_str("file_path").unwrap_or(""))
            .with_field("fn_name", payload.get_str("fn_name").unwrap_or(""))
            .with_field("body", body)
            .with_field("selected", if description.len() > 0 { "1" } else { "0" })
            .with_field("description", description);

        collection.update(id, vec, updated_payload).await?;
        println!("  [{}/{}] described chunk {}", idx + 1, total, id);
    }

    collection.flush().await?;
    println!("Done generating descriptions.");
    Ok(())
}
