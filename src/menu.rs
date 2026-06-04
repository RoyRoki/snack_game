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
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let result = loop {
        let mode = select_mode(&mut stdout);
        if let Some(diff) = select_diff(&mut stdout, mode) {
            break (mode, diff);
        }
        // Esc pressed — loop back to mode selection
    };

    terminal::disable_raw_mode().unwrap();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).unwrap();
    result
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

    let lore = [
        "You are the Ancient Snake, guardian of the Digital Grid.",
        "Consume the sacred Bits. Grow in power. Never stop moving.",
    ];
    for (i, line) in lore.iter().enumerate() {
        let _ = queue!(
            stdout, crossterm::cursor::MoveTo(2, 9 + i as u16),
            SetForegroundColor(Color::DarkGrey),
            Print(line), ResetColor
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
            Print("Select Mode  [↑/↓] Nav  [Enter] Select  [A] Achievements  [Q] Quit"),
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
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    show_achievements_screen(&mut *stdout);
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

fn select_diff(stdout: &mut impl Write, mode: Mode) -> Option<Diff> {
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
                    return Some(DIFFS[selected]);
                }
                KeyCode::Esc => {
                    return None;
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
            let name_display = if entry.name.is_empty() { "---".to_string() } else { entry.name.clone() };
            let _ = queue!(
                stdout,
                crossterm::cursor::MoveTo(4, start_row + 1 + i as u16),
                SetForegroundColor(Color::White),
                Print(format!("{}. {} {:6}pts  Lvl {}", i + 1, name_display, entry.score, entry.level)),
                ResetColor
            );
        }
    }

    let _ = queue!(
        stdout, crossterm::cursor::MoveTo(2, start_row + 7),
        SetForegroundColor(Color::DarkGrey),
        Print("─────────────────────────────────"),
        ResetColor
    );
    let daily = storage::get_daily();
    let daily_color = if daily.completed { Color::Green } else { Color::Yellow };
    let _ = queue!(
        stdout, crossterm::cursor::MoveTo(2, start_row + 8),
        SetForegroundColor(daily_color),
        Print(format!("Daily: {}", daily.description)),
        ResetColor
    );
    let _ = queue!(
        stdout, crossterm::cursor::MoveTo(2, start_row + 9),
        SetForegroundColor(Color::DarkGrey),
        Print(format!("Progress: {}/{}{}", daily.progress, daily.target,
            if daily.completed { " ✓" } else { "" })),
        ResetColor
    );
}

fn show_achievements_screen(stdout: &mut impl Write) {
    loop {
        let _ = queue!(stdout, Clear(ClearType::All));
        let _ = queue!(
            stdout, crossterm::cursor::MoveTo(2, 1),
            SetForegroundColor(Color::Yellow),
            Print("  ACHIEVEMENTS  "),
            ResetColor
        );
        for (i, &a) in crate::types::Achievement::ALL.iter().enumerate() {
            let unlocked = storage::is_unlocked(a);
            let row = 3 + i as u16 * 2;
            let (sym, color) = if unlocked { ("✓", Color::Green) } else { ("○", Color::DarkGrey) };
            let _ = queue!(
                stdout, crossterm::cursor::MoveTo(2, row),
                SetForegroundColor(color),
                Print(format!("{} {} — {}", sym, a.name(), a.description())),
                ResetColor
            );
        }
        let _ = queue!(
            stdout, crossterm::cursor::MoveTo(2, 24),
            SetForegroundColor(Color::DarkGrey),
            Print("[Any key] Back"),
            ResetColor
        );
        let _ = stdout.flush();
        if event::read().is_ok() { break; }
    }
}

/// Shows a 3-char initials entry screen and returns the entered name.
pub fn enter_initials(stdout: &mut impl Write, score: u32) -> String {
    let mut chars = [b'A', b'A', b'A'];
    let mut cursor = 0usize;

    loop {
        let _ = queue!(stdout, Clear(ClearType::All));
        let _ = queue!(
            stdout, crossterm::cursor::MoveTo(2, 5),
            SetForegroundColor(Color::Yellow),
            Print(format!("Score: {}  —  Enter your initials:", score)),
            ResetColor
        );
        // Draw the 3 chars
        for (i, &ch) in chars.iter().enumerate() {
            let col = 10 + i as u16 * 4;
            let color = if i == cursor { Color::Green } else { Color::White };
            let _ = queue!(
                stdout, crossterm::cursor::MoveTo(col, 7),
                SetForegroundColor(Color::Black),
                crossterm::style::SetBackgroundColor(color),
                Print(format!(" {} ", ch as char)),
                ResetColor
            );
        }
        let _ = queue!(
            stdout, crossterm::cursor::MoveTo(2, 9),
            SetForegroundColor(Color::DarkGrey),
            Print("[↑/↓] Change letter   [←/→] Move   [Enter] Confirm"),
            ResetColor
        );
        let _ = stdout.flush();

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    chars[cursor] = if chars[cursor] == b'Z' { b'A' } else { chars[cursor] + 1 };
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    chars[cursor] = if chars[cursor] == b'A' { b'Z' } else { chars[cursor] - 1 };
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    if cursor < 2 { cursor += 1; }
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    if cursor > 0 { cursor -= 1; }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    return String::from_utf8_lossy(&chars).to_string();
                }
                KeyCode::Esc => {
                    return "---".to_string();
                }
                _ => {}
            }
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
