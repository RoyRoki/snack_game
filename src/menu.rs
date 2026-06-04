use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::storage;
use crate::types::*;

const MODES: &[Mode] = &[Mode::Classic, Mode::Portal, Mode::Maze, Mode::TimeAttack];
const DIFFS: &[Diff] = &[Diff::Nokiya, Diff::Easy, Diff::Normal, Diff::Hard, Diff::Insane];

pub fn run_menu() -> (Mode, Diff) {
    let mut stdout = io::stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide
    )
    .unwrap();
    terminal::enable_raw_mode().unwrap();

    let mode = select_mode(&mut stdout);
    let diff = select_diff(&mut stdout, mode);

    terminal::disable_raw_mode().unwrap();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).unwrap();

    (mode, diff)
}

fn draw_banner(stdout: &mut impl Write) {
    let _ = queue!(stdout, Clear(ClearType::All));
    let lines = [
        r"  ____              _          ",
        r" / ___| _ __   __ _| | _____   ",
        r" \___ \| '_ \ / _` | |/ / _ \  ",
        r"  ___) | | | | (_| |   <  __/  ",
        r" |____/|_| |_|\__,_|_|\_\___|  ",
        r"                                ",
        r"       N O K I A   S N A K E   ",
    ];

    for (i, line) in lines.iter().enumerate() {
        let _ = queue!(
            stdout,
            crossterm::cursor::MoveTo(2, i as u16 + 1),
            SetForegroundColor(Color::Green),
            Print(line),
            ResetColor
        );
    }
}

fn select_mode(stdout: &mut impl Write) -> Mode {
    let mut selected = 0usize;

    loop {
        draw_banner(stdout);

        let _ = queue!(
            stdout,
            crossterm::cursor::MoveTo(2, 10),
            SetForegroundColor(Color::White),
            Print("Select Game Mode  [↑/↓ or W/S] Navigate  [Enter/Space] Select"),
            ResetColor
        );

        for (i, &mode) in MODES.iter().enumerate() {
            let row = 12 + i as u16 * 3;
            if i == selected {
                let _ = queue!(
                    stdout,
                    crossterm::cursor::MoveTo(2, row),
                    SetForegroundColor(Color::Black),
                    crossterm::style::SetBackgroundColor(Color::Green),
                    Print(format!("  ► {}  ", mode.name())),
                    ResetColor
                );
            } else {
                let _ = queue!(
                    stdout,
                    crossterm::cursor::MoveTo(2, row),
                    SetForegroundColor(Color::White),
                    Print(format!("    {}  ", mode.name())),
                    ResetColor
                );
            }
            let _ = queue!(
                stdout,
                crossterm::cursor::MoveTo(4, row + 1),
                SetForegroundColor(Color::DarkGrey),
                Print(mode.description()),
                ResetColor
            );
        }

        // Show high scores for currently selected mode + Normal diff
        draw_preview_scores(stdout, MODES[selected], Diff::Normal, 24);

        let _ = stdout.flush();

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = MODES.len() - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    selected = (selected + 1) % MODES.len();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    return MODES[selected];
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    // Cleanup and exit
                    let _ = terminal::disable_raw_mode();
                    let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

fn select_diff(stdout: &mut impl Write, mode: Mode) -> Diff {
    let mut selected = 2usize; // Default: Normal

    loop {
        draw_banner(stdout);

        let _ = queue!(
            stdout,
            crossterm::cursor::MoveTo(2, 10),
            SetForegroundColor(Color::White),
            Print(format!(
                "Select Difficulty for {}  [↑/↓ or W/S] Navigate  [Enter/Space] Select",
                mode.name()
            )),
            ResetColor
        );

        for (i, &diff) in DIFFS.iter().enumerate() {
            let row = 12 + i as u16 * 2;
            let info = format!(
                "{}ms  x{:.0} multiplier  len={}  obstacles={}",
                diff.tick_ms(),
                diff.multiplier(),
                diff.init_len(),
                diff.obstacle_count()
            );

            if i == selected {
                let _ = queue!(
                    stdout,
                    crossterm::cursor::MoveTo(2, row),
                    SetForegroundColor(Color::Black),
                    crossterm::style::SetBackgroundColor(Color::Green),
                    Print(format!("  ► {:8}  {}", diff.name(), info)),
                    ResetColor
                );
            } else {
                let _ = queue!(
                    stdout,
                    crossterm::cursor::MoveTo(2, row),
                    SetForegroundColor(Color::White),
                    Print(format!("    {:8}  {}", diff.name(), info)),
                    ResetColor
                );
            }
        }

        // Show high scores for current combo
        draw_preview_scores(stdout, mode, DIFFS[selected], 24);

        let _ = stdout.flush();

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = DIFFS.len() - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    selected = (selected + 1) % DIFFS.len();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    return DIFFS[selected];
                }
                KeyCode::Esc => {
                    // Go back to mode selection — we handle this by just returning Normal
                    return DIFFS[selected];
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    let _ = terminal::disable_raw_mode();
                    let _ = execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen);
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

fn draw_preview_scores(stdout: &mut impl Write, mode: Mode, diff: Diff, start_row: u16) {
    let scores = storage::get_top_scores(mode, diff);
    let _ = queue!(
        stdout,
        crossterm::cursor::MoveTo(2, start_row),
        SetForegroundColor(Color::Yellow),
        Print(format!("Top Scores — {} / {}:", mode.name(), diff.name())),
        ResetColor
    );

    if scores.is_empty() {
        let _ = queue!(
            stdout,
            crossterm::cursor::MoveTo(4, start_row + 1),
            SetForegroundColor(Color::DarkGrey),
            Print("No scores yet"),
            ResetColor
        );
    } else {
        for (i, entry) in scores.iter().enumerate() {
            let _ = queue!(
                stdout,
                crossterm::cursor::MoveTo(4, start_row + 1 + i as u16),
                SetForegroundColor(Color::White),
                Print(format!(
                    "{}. {:6}pts  Lvl {}",
                    i + 1,
                    entry.score,
                    entry.level
                )),
                ResetColor
            );
        }
    }
}

#[allow(dead_code)]
pub fn show_high_scores(mode: Mode, diff: Diff) {
    let scores = storage::get_top_scores(mode, diff);
    println!("--- High Scores: {} / {} ---", mode.name(), diff.name());
    if scores.is_empty() {
        println!("  No scores yet.");
    } else {
        for (i, entry) in scores.iter().enumerate() {
            println!("  {}. {} pts (Level {})", i + 1, entry.score, entry.level);
        }
    }
}
