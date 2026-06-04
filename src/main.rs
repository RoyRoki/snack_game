use clap::Parser;

mod game;
mod menu;
mod render;
mod storage;
mod types;

use types::{Diff, Mode};

#[derive(Parser, Debug)]
#[command(name = "snack_game", about = "Nokia Snake Game")]
struct Args {
    /// Skip menu and start immediately
    #[arg(long)]
    start: bool,

    /// Game mode: classic, portal, maze, timeattack
    #[arg(long, default_value = "classic")]
    mode: String,

    /// Difficulty: nokiya, easy, normal, hard, insane
    #[arg(long, default_value = "normal")]
    difficulty: String,
}

fn parse_mode(s: &str) -> Mode {
    match s.to_lowercase().as_str() {
        "classic" => Mode::Classic,
        "portal" => Mode::Portal,
        "maze" => Mode::Maze,
        "timeattack" | "time_attack" | "time-attack" => Mode::TimeAttack,
        _ => Mode::Classic,
    }
}

fn parse_diff(s: &str) -> Diff {
    match s.to_lowercase().as_str() {
        "nokiya" => Diff::Nokiya,
        "easy" => Diff::Easy,
        "normal" => Diff::Normal,
        "hard" => Diff::Hard,
        "insane" => Diff::Insane,
        _ => Diff::Normal,
    }
}

fn main() {
    let args = Args::parse();

    let (mode, diff) = if args.start {
        (parse_mode(&args.mode), parse_diff(&args.difficulty))
    } else {
        menu::run_menu()
    };

    game::run(mode, diff);
}
