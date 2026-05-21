use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::distractors::QuizEntry;

#[derive(Serialize)]
struct QuizItemJson<'a> {
    code: &'a str,
    correct: &'a str,
    distractors: [&'a str; 3],
}

pub fn export_json(entries: &[QuizEntry], output_path: &Path) -> Result<()> {
    let items: Vec<QuizItemJson> = entries
        .iter()
        .map(|e| QuizItemJson {
            code: &e.body,
            correct: &e.correct,
            distractors: [
                &e.distractors[0],
                &e.distractors[1],
                &e.distractors[2],
            ],
        })
        .collect();

    let json = serde_json::to_string_pretty(&items)?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(output_path, json)?;
    println!("Exported {} quiz items to {}", items.len(), output_path.display());
    Ok(())
}
