use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::types::{Achievement, Diff, Mode};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScoreEntry {
    pub score: u32,
    pub level: u32,
    pub timestamp: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerProfile {
    pub total_games: u32,
    pub total_score: u64,
    pub total_foods: u64,
    pub best_streak: u32,
    pub achievements: Vec<String>,
}

impl PlayerProfile {
    pub fn has(&self, a: Achievement) -> bool {
        self.achievements.iter().any(|s| s == a.id())
    }
    pub fn unlock(&mut self, a: Achievement) -> bool {
        if self.has(a) { return false; }
        self.achievements.push(a.id().to_string());
        true
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DailyMission {
    pub date: String,
    pub description: String,
    pub target_type: String,  // "foods" | "score" | "streak"
    pub target_mode: String,  // mode name or "any"
    pub target: u32,
    pub progress: u32,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GameData {
    pub scores: HashMap<String, Vec<ScoreEntry>>,
    pub profile: PlayerProfile,
    pub daily: Option<DailyMission>,
}

// --- persistence ---

fn data_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home).join(".snake_scores.json")
}

pub fn load_data() -> GameData {
    let path = data_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_data(data: &GameData) {
    let path = data_path();
    if let Ok(s) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(path, s);
    }
}

// --- daily mission ---

fn today_str() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0);
    let (y, m, d) = epoch_secs_to_ymd(secs);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn epoch_secs_to_ymd(secs: u64) -> (u64, u64, u64) {
    let mut days = secs / 86400;
    let mut year = 1970u64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let dy = if leap { 366u64 } else { 365 };
        if days < dy { break; }
        days -= dy;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days = [31u64, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md { break; }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn daily_seed(date: &str) -> u64 {
    date.bytes().fold(5381u64, |h, b| h.wrapping_mul(33).wrapping_add(b as u64))
}

fn make_daily(date: &str) -> DailyMission {
    let idx = (daily_seed(date) as usize) % MISSIONS.len();
    let &(tt, tm, target, desc) = &MISSIONS[idx];
    DailyMission {
        date: date.to_string(),
        description: desc.to_string(),
        target_type: tt.to_string(),
        target_mode: tm.to_string(),
        target,
        progress: 0,
        completed: false,
    }
}

const MISSIONS: &[(&str, &str, u32, &str)] = &[
    ("foods",  "Classic",    15, "Eat 15 foods in Classic mode"),
    ("foods",  "Portal",     10, "Eat 10 foods in Portal mode"),
    ("foods",  "Maze",       12, "Eat 12 foods in Maze mode"),
    ("streak", "any",         8, "Reach a streak of 8"),
    ("score",  "Classic",   120, "Score 120 pts in Classic"),
    ("score",  "Portal",    150, "Score 150 pts in Portal"),
    ("streak", "any",        10, "Reach a streak of 10"),
    ("foods",  "TimeAttack",  8, "Eat 8 foods in Time Attack"),
    ("score",  "any",       200, "Score 200 pts in any mode"),
    ("foods",  "any",        20, "Eat 20 foods in one game"),
];

pub fn get_daily() -> DailyMission {
    let mut data = load_data();
    let today = today_str();
    let fresh = match &data.daily {
        None => true,
        Some(d) => d.date != today,
    };
    if fresh {
        data.daily = Some(make_daily(&today));
        save_data(&data);
    }
    data.daily.unwrap()
}

pub fn update_daily_progress(mode: Mode, ttype: &str, value: u32) {
    let mut data = load_data();
    let today = today_str();
    let fresh = match &data.daily { None => true, Some(d) => d.date != today };
    if fresh { data.daily = Some(make_daily(&today)); }
    if let Some(d) = &mut data.daily {
        if !d.completed {
            let mode_ok = d.target_mode == "any" || d.target_mode == mode.name();
            let type_ok = d.target_type == ttype;
            if mode_ok && type_ok {
                d.progress = d.progress.max(value);
                if d.progress >= d.target { d.completed = true; }
            }
        }
    }
    save_data(&data);
}

// --- scores / profile ---

pub fn score_key(mode: Mode, diff: Diff) -> String {
    format!("{}_{}", mode.name(), diff.name())
}

pub fn get_top_scores(mode: Mode, diff: Diff) -> Vec<ScoreEntry> {
    let key = score_key(mode, diff);
    load_data().scores.get(&key).cloned().unwrap_or_default()
}

pub fn get_best_score(mode: Mode, diff: Diff) -> u32 {
    get_top_scores(mode, diff).first().map(|e| e.score).unwrap_or(0)
}

pub fn record_score(mode: Mode, diff: Diff, score: u32, level: u32) {
    record_score_named(mode, diff, score, level, "");
}

pub fn record_score_named(mode: Mode, diff: Diff, score: u32, level: u32, name: &str) {
    let mut data = load_data();
    let key = score_key(mode, diff);
    let entry = ScoreEntry {
        score, level, name: name.to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0),
    };
    let list = data.scores.entry(key).or_default();
    list.push(entry);
    list.sort_by(|a, b| b.score.cmp(&a.score));
    list.truncate(5);
    data.profile.total_games += 1;
    data.profile.total_score += score as u64;
    save_data(&data);
}

pub fn update_profile_stats(foods_delta: u64, max_streak: u32) {
    let mut data = load_data();
    data.profile.total_foods += foods_delta;
    if max_streak > data.profile.best_streak {
        data.profile.best_streak = max_streak;
    }
    save_data(&data);
}

pub fn get_profile() -> PlayerProfile {
    load_data().profile
}

/// Unlocks an achievement; returns true if it was newly unlocked.
/// Also auto-unlocks Collector if all others are done.
pub fn unlock_achievement(a: Achievement) -> bool {
    let mut data = load_data();
    let newly = data.profile.unlock(a);
    if newly {
        let all_others_done = Achievement::ALL.iter()
            .filter(|&&x| x != Achievement::Collector)
            .all(|&x| data.profile.has(x));
        if all_others_done { data.profile.unlock(Achievement::Collector); }
        save_data(&data);
    }
    newly
}

pub fn is_unlocked(a: Achievement) -> bool {
    load_data().profile.has(a)
}
