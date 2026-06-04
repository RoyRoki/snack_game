use std::io::Write;

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::types::*;
use crate::game::GameState;

// Layout constants
// Row 0-1: HUD
// Row 2: top border
// Row 3..22: grid cells (20 rows)
// Row 23: bottom border
// Row 24+: status bar

const HUD_ROWS: u16 = 2;
const BORDER_TOP_ROW: u16 = HUD_ROWS;
const GRID_START_ROW: u16 = HUD_ROWS + 1;
const BORDER_BOT_ROW: u16 = GRID_START_ROW + GRID_H as u16;
const STATUS_ROW: u16 = BORDER_BOT_ROW + 1;

// Left border at col 0, cells start at col 1, each cell is 2 wide
// Right border at col 1 + 20*2 = 41
const GRID_LEFT_COL: u16 = 1;
const CELL_W: u16 = 2;
const GRID_COLS: u16 = GRID_W as u16 * CELL_W;
const BORDER_RIGHT_COL: u16 = GRID_LEFT_COL + GRID_COLS;

pub fn draw(stdout: &mut impl Write, state: &GameState) {
    // Check terminal size
    let (cols, rows) = terminal::size().unwrap_or((80, 30));
    if cols < 50 || rows < 30 {
        let _ = queue!(
            stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            SetForegroundColor(Color::Red),
            Print("Terminal too small! Need at least 50 cols x 30 rows."),
            ResetColor
        );
        let _ = stdout.flush();
        return;
    }

    let _ = queue!(stdout, Clear(ClearType::All));

    draw_hud(stdout, state);
    draw_border(stdout, state.danger_next_tick);
    draw_grid(stdout, state);
    draw_status(stdout, state);

    match state.status {
        Status::Paused => draw_overlay_paused(stdout),
        Status::Over => draw_overlay_game_over(stdout, state),
        Status::Won => draw_overlay_win(stdout, state),
        Status::Running => {}
    }

    // CD2: Achievement popup
    if let Some((ref msg, _)) = state.pending_achievement {
        draw_achievement_popup(stdout, msg);
    }
    // CD7: Reversed controls indicator
    if state.reversed_until.map(|t| std::time::Instant::now() < t).unwrap_or(false) {
        draw_reverse_indicator(stdout);
    }

    let _ = stdout.flush();
}

fn draw_hud(stdout: &mut impl Write, state: &GameState) {
    // Row 0: Mode + Difficulty + Score
    let _ = queue!(stdout, MoveTo(0, 0));
    let _ = queue!(
        stdout,
        SetForegroundColor(Color::White),
        Print(format!(
            " {} | {}",
            state.mode.name(),
            state.diff.name()
        )),
        ResetColor
    );

    // Score
    let score_str = format!("Score: {}", state.score);
    let score_col = BORDER_RIGHT_COL.saturating_sub(score_str.len() as u16 + 1);
    let _ = queue!(
        stdout,
        MoveTo(score_col, 0),
        SetForegroundColor(Color::Yellow),
        Print(&score_str),
        ResetColor
    );

    // Personal best (CD4/CD8)
    if state.personal_best > 0 {
        let pb_str = format!("Best:{}", state.personal_best);
        let pb_col = BORDER_RIGHT_COL.saturating_sub(pb_str.len() as u16 + 1);
        let _ = queue!(stdout, MoveTo(pb_col, 1), SetForegroundColor(Color::DarkGrey), Print(&pb_str), ResetColor);
    }

    // Row 1: Lives (Time Attack) and Level
    let level_str = format!(" Level: {}", state.level);
    let _ = queue!(
        stdout,
        MoveTo(0, 1),
        SetForegroundColor(Color::Grey),
        Print(&level_str),
        ResetColor
    );

    // Streak multiplier indicator (CD3)
    let mult_str = if state.streak >= 10 {
        format!(" ×2.0 streak:{}", state.streak)
    } else if state.streak >= 5 {
        format!(" ×1.5 streak:{}", state.streak)
    } else if state.streak > 0 {
        format!(" streak:{}", state.streak)
    } else {
        String::new()
    };
    if !mult_str.is_empty() {
        let _ = queue!(stdout, MoveTo(0, 1), SetForegroundColor(Color::Yellow), Print(&mult_str), ResetColor);
    }

    if state.mode == Mode::TimeAttack {
        // Lives
        let lives_str = format!("Lives: {}", "♥ ".repeat(state.lives as usize));
        let lives_col = BORDER_RIGHT_COL.saturating_sub(lives_str.len() as u16 + 1);
        let _ = queue!(
            stdout,
            MoveTo(lives_col, 1),
            SetForegroundColor(Color::Red),
            Print(&lives_str),
            ResetColor
        );
    } else if state.mode == Mode::Portal {
        let target_str = format!("Target: {}", state.portal_target);
        let target_col = BORDER_RIGHT_COL.saturating_sub(target_str.len() as u16 + 1);
        let _ = queue!(
            stdout,
            MoveTo(target_col, 1),
            SetForegroundColor(Color::Cyan),
            Print(&target_str),
            ResetColor
        );
    } else if state.mode == Mode::Maze {
        let food_str = format!(
            "Food: {}/{}",
            state.maze_total_food.saturating_sub(state.food.len()),
            state.maze_total_food
        );
        let food_col = BORDER_RIGHT_COL.saturating_sub(food_str.len() as u16 + 1);
        let _ = queue!(
            stdout,
            MoveTo(food_col, 1),
            SetForegroundColor(Color::Green),
            Print(&food_str),
            ResetColor
        );
    }
}

fn draw_border(stdout: &mut impl Write, danger: bool) {
    let border_color = if danger { Color::Red } else { Color::White };
    let _ = queue!(stdout, SetForegroundColor(border_color));

    // Top border
    let _ = queue!(stdout, MoveTo(0, BORDER_TOP_ROW), Print("┌"));
    for _ in 0..GRID_COLS {
        let _ = queue!(stdout, Print("─"));
    }
    let _ = queue!(stdout, Print("┐"));

    // Side borders
    for row in 0..GRID_H as u16 {
        let _ = queue!(stdout, MoveTo(0, GRID_START_ROW + row), Print("│"));
        let _ = queue!(
            stdout,
            MoveTo(BORDER_RIGHT_COL, GRID_START_ROW + row),
            Print("│")
        );
    }

    // Bottom border
    let _ = queue!(stdout, MoveTo(0, BORDER_BOT_ROW), Print("└"));
    for _ in 0..GRID_COLS {
        let _ = queue!(stdout, Print("─"));
    }
    let _ = queue!(stdout, Print("┘"));

    let _ = queue!(stdout, ResetColor);
}

fn draw_grid(stdout: &mut impl Write, state: &GameState) {
    // Build cell grid
    let snake_head = state.snake.front().copied();
    let snake_set: std::collections::HashSet<Pos> = state.snake.iter().copied().collect();

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let pos = Pos::new(x, y);
            let col = GRID_LEFT_COL + x as u16 * CELL_W;
            let row = GRID_START_ROW + y as u16;

            let _ = queue!(stdout, MoveTo(col, row));

            if state.walls.contains(&pos) {
                let _ = queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print("▓ "),
                    ResetColor
                );
            } else if snake_head == Some(pos) {
                let _ = queue!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("◉ "),
                    ResetColor
                );
            } else if snake_set.contains(&pos) {
                // Determine if tail
                let is_tail = state.snake.back() == Some(&pos) && state.snake.len() > 1;
                if is_tail {
                    let _ = queue!(
                        stdout,
                        SetForegroundColor(Color::DarkGreen),
                        Print("▪ "),
                        ResetColor
                    );
                } else {
                    let _ = queue!(
                        stdout,
                        SetForegroundColor(Color::DarkGreen),
                        Print("█ "),
                        ResetColor
                    );
                }
            } else if let Some(food) = state.food.iter().find(|f| f.pos == pos) {
                let color = match food.kind {
                    FoodKind::Regular => Color::Red,
                    FoodKind::Bonus => Color::Magenta,
                    FoodKind::Golden => Color::Yellow,
                    FoodKind::Shrink => Color::Cyan,
                    FoodKind::Mystery => Color::White,
                };
                let sym = format!("{} ", food.kind.symbol());
                let _ = queue!(
                    stdout,
                    SetForegroundColor(color),
                    Print(&sym),
                    ResetColor
                );
            } else {
                let _ = queue!(stdout, Print("  "));
            }
        }
    }

    // Floating score text at eaten position (CD3)
    if let Some((pos, ref txt, t)) = state.last_eaten {
        if t.elapsed().as_secs_f64() < 0.6 {
            let col = (GRID_LEFT_COL + pos.x as u16 * CELL_W)
                .min(BORDER_RIGHT_COL.saturating_sub(txt.len() as u16 + 1));
            let show_row = if pos.y > 0 {
                GRID_START_ROW + pos.y as u16 - 1
            } else {
                GRID_START_ROW
            };
            let _ = queue!(stdout, MoveTo(col, show_row), SetForegroundColor(Color::Yellow), Print(txt.as_str()), ResetColor);
        }
    }
}

fn draw_status(stdout: &mut impl Write, state: &GameState) {
    let row = STATUS_ROW;

    // Clear status rows
    let _ = queue!(stdout, MoveTo(0, row), Print(" ".repeat(BORDER_RIGHT_COL as usize + 2)));
    let _ = queue!(stdout, MoveTo(0, row + 1), Print(" ".repeat(BORDER_RIGHT_COL as usize + 2)));

    // Time Attack timers
    if state.mode == Mode::TimeAttack {
        let food_rem = state.food_timer_remaining_secs();
        let game_rem = state.time_attack_remaining_secs();

        let food_color = if food_rem < 3.0 { Color::Red } else { Color::Green };
        let game_color = if game_rem < 10.0 { Color::Red } else { Color::Green };

        let _ = queue!(
            stdout,
            MoveTo(0, row),
            SetForegroundColor(food_color),
            Print(format!(" Food: {:.1}s  ", food_rem)),
            ResetColor,
            SetForegroundColor(game_color),
            Print(format!("Game: {:.1}s", game_rem)),
            ResetColor
        );
    }

    // Streak message
    if let Some((msg, _)) = &state.message {
        let msg_col = BORDER_RIGHT_COL.saturating_sub(msg.len() as u16 + 1);
        let _ = queue!(
            stdout,
            MoveTo(msg_col, row),
            SetForegroundColor(Color::Yellow),
            Print(msg),
            ResetColor
        );
    }

    // Food expiry countdowns (CD6)
    let expiring: Vec<(&Food, f64)> = state.food.iter()
        .filter_map(|f| f.remaining_secs().map(|r| (f, r)))
        .collect();
    if !expiring.is_empty() {
        let mut expiry_str = String::new();
        for (f, r) in &expiring {
            expiry_str.push_str(&format!(" {}:{:.1}s", f.kind.symbol(), r));
        }
        let _ = queue!(
            stdout, MoveTo(0, row + 2),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("Expiring:{}", expiry_str)),
            ResetColor
        );
    }

    // Streak-lost alert (CD8)
    if let Some(t) = state.streak_lost_at {
        if t.elapsed().as_secs_f64() < 1.5 {
            let _ = queue!(
                stdout, MoveTo(BORDER_RIGHT_COL.saturating_sub(14), row),
                SetForegroundColor(Color::Red),
                Print("Streak lost!"),
                ResetColor
            );
        }
    }

    // Reversed controls warning (CD7) — supplement the top-left indicator
    if state.reversed_until.map(|t| std::time::Instant::now() < t).unwrap_or(false) {
        let _ = queue!(
            stdout, MoveTo(0, row + 1),
            SetForegroundColor(Color::Magenta),
            Print(" ↔ CONTROLS REVERSED — up=down, left=right ↔ "),
            ResetColor
        );
    }

    // Daily mission progress hint
    let mission_color = if state.daily_completed { Color::Green } else { Color::DarkGrey };
    let mission_str = if state.daily_completed {
        format!(" ★ Daily done: {} ✓", &state.daily_desc)
    } else {
        format!(" Daily: {} ({}/{})", &state.daily_desc, state.daily_progress, state.daily_target)
    };
    // Truncate to avoid overflow
    let max_len = (BORDER_RIGHT_COL as usize + 1).min(60);
    let mission_str: String = mission_str.chars().take(max_len).collect();
    let _ = queue!(stdout, MoveTo(0, row + 3), SetForegroundColor(mission_color), Print(&mission_str), ResetColor);

    // Controls hint
    let hint = " [←↑→↓/WASD] Move  [P] Pause  [Q/Esc] Quit  [R] Restart";
    let _ = queue!(
        stdout,
        MoveTo(0, row + 1),
        SetForegroundColor(Color::DarkGrey),
        Print(hint),
        ResetColor
    );
}

fn draw_overlay_paused(stdout: &mut impl Write) {
    let msg = "  PAUSED — Press P to resume  ";
    let msg_len = msg.len() as u16;
    let col = (BORDER_RIGHT_COL / 2).saturating_sub(msg_len / 2);
    let row = GRID_START_ROW + GRID_H as u16 / 2;
    let _ = queue!(
        stdout,
        MoveTo(col, row),
        SetForegroundColor(Color::Black),
        crossterm::style::SetBackgroundColor(Color::White),
        Print(msg),
        ResetColor
    );
}

fn draw_overlay_game_over(stdout: &mut impl Write, state: &GameState) {
    let row = GRID_START_ROW + GRID_H as u16 / 2 - 2;
    let lines = [
        format!(" GAME OVER "),
        format!(" Score: {}  Best: {} ", state.score, state.personal_best),
        format!(" Foods: {}  Max streak: {} ", state.foods_eaten, state.max_streak),
        format!(" [R] Restart   [Q] Quit "),
    ];
    for (i, line) in lines.iter().enumerate() {
        let col = (BORDER_RIGHT_COL / 2).saturating_sub(line.len() as u16 / 2).max(1);
        let bg = if i == 0 { Color::Red } else { Color::DarkRed };
        let _ = queue!(
            stdout, MoveTo(col, row + i as u16),
            SetForegroundColor(Color::White),
            crossterm::style::SetBackgroundColor(bg),
            Print(line), ResetColor
        );
    }
}

fn draw_overlay_win(stdout: &mut impl Write, state: &GameState) {
    let msg = format!(" YOU WIN! — Score: {} — [R] Next Level  [Q] Quit ", state.score);
    let msg_len = msg.len() as u16;
    let col = (BORDER_RIGHT_COL / 2).saturating_sub(msg_len / 2).max(1);
    let row = GRID_START_ROW + GRID_H as u16 / 2;
    let _ = queue!(
        stdout,
        MoveTo(col, row),
        SetForegroundColor(Color::Black),
        crossterm::style::SetBackgroundColor(Color::Green),
        Print(&msg),
        ResetColor
    );
}

fn draw_achievement_popup(stdout: &mut impl Write, msg: &str) {
    let display = format!("  {}  ", msg);
    let col = (BORDER_RIGHT_COL / 2).saturating_sub(display.len() as u16 / 2).max(1);
    let row = GRID_START_ROW + 1;
    let _ = queue!(
        stdout, MoveTo(col, row),
        SetForegroundColor(Color::Black),
        crossterm::style::SetBackgroundColor(Color::Yellow),
        Print(&display), ResetColor
    );
}

fn draw_reverse_indicator(stdout: &mut impl Write) {
    let _ = queue!(
        stdout, MoveTo(1, BORDER_TOP_ROW + 1),
        SetForegroundColor(Color::Magenta),
        Print("↔REVERSED"),
        ResetColor
    );
}
