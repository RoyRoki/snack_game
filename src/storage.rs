use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::types::{Diff, Mode};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreEntry {
    pub score: u32,
    pub level: u32,
    pub timestamp: u64,
}

pub fn score_key(mode: Mode, diff: Diff) -> String {
    format!("{}_{}", mode.name(), diff.name())
}

fn scores_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home).join(".snake_scores.json")
}

pub fn load_scores() -> HashMap<String, Vec<ScoreEntry>> {
    let path = scores_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

pub fn save_scores(scores: &HashMap<String, Vec<ScoreEntry>>) {
    let path = scores_path();
    if let Ok(contents) = serde_json::to_string_pretty(scores) {
        let _ = std::fs::write(path, contents);
    }
}

pub fn record_score(mode: Mode, diff: Diff, score: u32, level: u32) {
    let mut scores = load_scores();
    let key = score_key(mode, diff);
    let entry = ScoreEntry {
        score,
        level,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };
    let list = scores.entry(key).or_default();
    list.push(entry);
    list.sort_by(|a, b| b.score.cmp(&a.score));
    list.truncate(5);
    save_scores(&scores);
}

pub fn get_top_scores(mode: Mode, diff: Diff) -> Vec<ScoreEntry> {
    let scores = load_scores();
    let key = score_key(mode, diff);
    scores.get(&key).cloned().unwrap_or_default()
}
