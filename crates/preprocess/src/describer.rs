use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::ollama;
use rusqlite::{Connection, params};

const LLM_MODEL: &str = "phi4-mini:3.8b";

const SYSTEM_PROMPT: &str = "\
You are a Rust code analyst. When given a Rust function body, respond with exactly one \
sentence describing what the code does functionally. Do not mention variable names, \
function names, or implementation details — only the purpose. \
Do not include any preamble, explanation, or punctuation beyond the single sentence.";

pub async fn describe_selected_chunks(conn: &Connection) -> Result<()> {
    let client = ollama::Client::new();
    let agent = client
        .agent(LLM_MODEL)
        .preamble(SYSTEM_PROMPT)
        .build();

    let mut stmt =
        conn.prepare("SELECT id, body FROM chunks WHERE selected = 1 AND description IS NULL")?;
    let rows: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let total = rows.len();
    println!("Generating descriptions for {} selected chunks...", total);

    for (idx, (id, body)) in rows.iter().enumerate() {
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

        conn.execute(
            "UPDATE chunks SET description = ?1 WHERE id = ?2",
            params![description, id],
        )?;

        println!("  [{}/{}] described chunk {}", idx + 1, total, id);
    }

    println!("Done generating descriptions.");
    Ok(())
}
