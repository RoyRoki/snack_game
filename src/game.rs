use std::collections::{HashSet, VecDeque};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor, event,
    event::{Event, KeyCode},
    execute, terminal,
};
use rand::Rng;

use crate::render;
use crate::storage;
use crate::types::*;

pub struct GameState {
    pub snake: VecDeque<Pos>,
    pub food: Vec<Food>,
    pub walls: HashSet<Pos>,
    pub dir: Dir,
    pub next_dir: Dir,
    pub score: u32,
    pub lives: u8,
    pub mode: Mode,
    pub diff: Diff,
    pub status: Status,
    pub foods_eaten: u32,
    pub streak: u32,
    pub level: u32,
    pub current_tick_ms: u64,
    // Time Attack
    pub food_timer: Option<Instant>,
    pub game_timer_start: Option<Instant>,
    // Golden food speed boost
    pub speed_boost_until: Option<Instant>,
    // Portal mode score target
    pub portal_target: u32,
    // Maze mode: total food to collect
    pub maze_total_food: usize,
    // Temporary message for UI
    pub message: Option<(String, Instant)>,
    // Insane mode idle ticks counter
    pub idle_ticks: u32,
}

impl GameState {
    pub fn new(mode: Mode, diff: Diff) -> Self {
        let mut state = GameState {
            snake: VecDeque::new(),
            food: Vec::new(),
            walls: HashSet::new(),
            dir: Dir::Right,
            next_dir: Dir::Right,
            score: 0,
            lives: 3,
            mode,
            diff,
            status: Status::Running,
            foods_eaten: 0,
            streak: 0,
            level: 1,
            current_tick_ms: diff.tick_ms(),
            food_timer: None,
            game_timer_start: None,
            speed_boost_until: None,
            portal_target: 50,
            maze_total_food: 0,
            message: None,
            idle_ticks: 0,
        };

        // Build initial snake at center
        let cx = GRID_W / 2;
        let cy = GRID_H / 2;
        let len = diff.init_len();
        for i in 0..len {
            state.snake.push_back(Pos::new(cx - i as i32, cy));
        }

        // Generate walls/obstacles based on mode
        match mode {
            Mode::Maze => {
                state.generate_maze();
                state.spawn_maze_food();
            }
            _ => {
                state.generate_obstacles();
                // Spawn initial food
                state.spawn_food(FoodKind::Regular);
            }
        }

        if mode == Mode::TimeAttack {
            state.food_timer = Some(Instant::now());
            state.game_timer_start = Some(Instant::now());
        }

        state
    }

    fn generate_obstacles(&mut self) {
        let count = self.diff.obstacle_count();
        if count == 0 {
            return;
        }

        let mut rng = rand::thread_rng();
        let cx = GRID_W / 2;
        let cy = GRID_H / 2;

        let mut placed = 0;
        let mut attempts = 0;
        while placed < count && attempts < 1000 {
            attempts += 1;
            let horizontal = rng.gen_bool(0.5);
            let seg_len = rng.gen_range(2..=5i32);

            let (sx, sy) = if horizontal {
                let x = rng.gen_range(1..GRID_W - seg_len - 1);
                let y = rng.gen_range(1..GRID_H - 1);
                (x, y)
            } else {
                let x = rng.gen_range(1..GRID_W - 1);
                let y = rng.gen_range(1..GRID_H - seg_len - 1);
                (x, y)
            };

            // Collect candidate positions
            let mut candidates = Vec::new();
            for i in 0..seg_len {
                let p = if horizontal {
                    Pos::new(sx + i, sy)
                } else {
                    Pos::new(sx, sy + i)
                };
                candidates.push(p);
            }

            // Check clearance from center (5x5 zone)
            let too_close = candidates.iter().any(|p| {
                (p.x - cx).abs() <= 3 && (p.y - cy).abs() <= 3
            });

            if too_close {
                continue;
            }

            // Check overlap with existing walls or snake
            let overlaps = candidates.iter().any(|p| {
                self.walls.contains(p) || self.snake.contains(p)
            });

            if overlaps {
                continue;
            }

            for p in candidates {
                self.walls.insert(p);
            }
            placed += 1;
        }
    }

    fn generate_maze(&mut self) {
        self.walls.clear();
        let mut rng = rand::thread_rng();
        let cx = GRID_W / 2;
        let cy = GRID_H / 2;

        let mut placed = 0;
        let mut attempts = 0;
        while placed < 15 && attempts < 2000 {
            attempts += 1;
            let horizontal = rng.gen_bool(0.5);
            let seg_len = rng.gen_range(2..=5i32);

            let (sx, sy) = if horizontal {
                let x = rng.gen_range(2..GRID_W - seg_len - 2);
                let y = rng.gen_range(2..GRID_H - 2);
                (x, y)
            } else {
                let x = rng.gen_range(2..GRID_W - 2);
                let y = rng.gen_range(2..GRID_H - seg_len - 2);
                (x, y)
            };

            let mut candidates = Vec::new();
            for i in 0..seg_len {
                let p = if horizontal {
                    Pos::new(sx + i, sy)
                } else {
                    Pos::new(sx, sy + i)
                };
                candidates.push(p);
            }

            // Keep 5x5 clear zone around center
            let too_close = candidates.iter().any(|p| {
                (p.x - cx).abs() <= 3 && (p.y - cy).abs() <= 3
            });

            if too_close {
                continue;
            }

            // Don't touch border
            let touches_border = candidates.iter().any(|p| {
                p.x <= 0 || p.x >= GRID_W - 1 || p.y <= 0 || p.y >= GRID_H - 1
            });

            if touches_border {
                continue;
            }

            let overlaps = candidates.iter().any(|p| self.walls.contains(p));
            if overlaps {
                continue;
            }

            for p in candidates {
                self.walls.insert(p);
            }
            placed += 1;
        }
    }

    fn spawn_maze_food(&mut self) {
        self.food.clear();
        // Scatter ~8 food items around
        for _ in 0..8 {
            self.spawn_food(FoodKind::Regular);
        }
        self.maze_total_food = self.food.len();
    }

    pub fn random_empty_pos(&self) -> Option<Pos> {
        let mut rng = rand::thread_rng();
        let snake_set: HashSet<Pos> = self.snake.iter().copied().collect();
        let food_set: HashSet<Pos> = self.food.iter().map(|f| f.pos).collect();

        let mut attempts = 0;
        loop {
            if attempts > 400 {
                return None;
            }
            let x = rng.gen_range(0..GRID_W);
            let y = rng.gen_range(0..GRID_H);
            let p = Pos::new(x, y);
            if !snake_set.contains(&p) && !food_set.contains(&p) && !self.walls.contains(&p) {
                return Some(p);
            }
            attempts += 1;
        }
    }

    pub fn spawn_food(&mut self, kind: FoodKind) {
        if let Some(pos) = self.random_empty_pos() {
            self.food.push(Food::new(pos, kind));
        }
    }

    pub fn effective_tick_ms(&self) -> u64 {
        if let Some(boost_until) = self.speed_boost_until {
            if Instant::now() < boost_until {
                return (self.current_tick_ms / 2).max(30);
            }
        }
        self.current_tick_ms
    }

    pub fn time_attack_remaining_secs(&self) -> f64 {
        if let Some(start) = self.game_timer_start {
            let elapsed = start.elapsed().as_secs_f64();
            (60.0 - elapsed).max(0.0)
        } else {
            0.0
        }
    }

    pub fn food_timer_remaining_secs(&self) -> f64 {
        if let Some(timer) = self.food_timer {
            let elapsed = timer.elapsed().as_secs_f64();
            (8.0 - elapsed).max(0.0)
        } else {
            8.0
        }
    }

    pub fn handle_dir(&mut self, dir: Dir) {
        if !self.dir.is_opposite(dir) {
            self.next_dir = dir;
        }
    }

    fn streak_bonus(&self) -> f64 {
        if self.streak >= 10 {
            2.0
        } else if self.streak >= 5 {
            1.5
        } else {
            1.0
        }
    }

    pub fn tick(&mut self) {
        if self.status != Status::Running {
            return;
        }

        // Remove expired food (Bonus, Golden, Shrink)
        self.food.retain(|f| !f.is_expired());

        // Apply direction
        self.dir = self.next_dir;
        let (dx, dy) = self.dir.delta();
        let head = *self.snake.front().unwrap();
        let mut new_head = Pos::new(head.x + dx, head.y + dy);

        // Handle bounds
        match self.mode {
            Mode::Portal => {
                // Wrap around
                new_head.x = new_head.x.rem_euclid(GRID_W);
                new_head.y = new_head.y.rem_euclid(GRID_H);
            }
            _ => {
                // Solid walls - check out of bounds
                if new_head.x < 0
                    || new_head.x >= GRID_W
                    || new_head.y < 0
                    || new_head.y >= GRID_H
                {
                    self.status = Status::Over;
                    return;
                }
            }
        }

        // Check wall collision
        if self.walls.contains(&new_head) {
            self.status = Status::Over;
            return;
        }

        // Check self collision (ignore tail since it will move)
        // We check against all body except tail (tail will be removed)
        let will_grow = self.food.iter().any(|f| f.pos == new_head);
        let check_len = if will_grow {
            self.snake.len()
        } else {
            self.snake.len() - 1
        };

        for i in 0..check_len {
            if self.snake[i] == new_head {
                self.status = Status::Over;
                return;
            }
        }

        // Move snake
        self.snake.push_front(new_head);

        // Check if ate food
        let eaten_idx = self.food.iter().position(|f| f.pos == new_head);

        if let Some(idx) = eaten_idx {
            let food = self.food.remove(idx);
            let grow = food.kind.grow();

            // Grow or shrink
            if grow >= 0 {
                // Keep tail (we already pushed head), add extra if grow > 1
                for _ in 1..grow {
                    let tail = *self.snake.back().unwrap();
                    self.snake.push_back(tail);
                }
            } else {
                // Shrink: pop tail plus additional abs(grow) cells
                self.snake.pop_back(); // normal tail removal
                for _ in 0..(-grow) {
                    if self.snake.len() > 1 {
                        self.snake.pop_back();
                    }
                }
            }

            // Scoring
            let base = food.kind.base_pts() as f64;
            let mult = self.diff.multiplier();
            let streak_b = self.streak_bonus();
            let mut pts = (base * mult * streak_b).round() as u32;

            // Time Attack bonus: +5 per second remaining
            if self.mode == Mode::TimeAttack {
                let remaining = self.food_timer_remaining_secs();
                pts += (remaining * 5.0).round() as u32;
                self.food_timer = Some(Instant::now());
            }

            self.score += pts;
            self.streak += 1;
            self.foods_eaten += 1;
            self.idle_ticks = 0;

            // Streak messages
            if self.streak == 5 {
                self.message = Some(("On a roll!".to_string(), Instant::now()));
            } else if self.streak == 10 {
                self.message = Some(("Unstoppable!".to_string(), Instant::now()));
            } else if self.streak > 10 && self.streak % 5 == 0 {
                self.message = Some(("Unstoppable!".to_string(), Instant::now()));
            }

            // Golden food speed boost
            if food.kind == FoodKind::Golden {
                self.speed_boost_until = Some(Instant::now() + Duration::from_secs(3));
            }

            // Mode-specific logic after eating
            match self.mode {
                Mode::Classic => {
                    // Speed increase every 5 foods
                    if self.foods_eaten % 5 == 0 {
                        self.current_tick_ms = self.current_tick_ms.saturating_sub(10).max(50);
                    }
                    // Respawn regular food
                    self.spawn_food(FoodKind::Regular);
                }
                Mode::Portal => {
                    // Respawn regular food
                    if !self.food.iter().any(|f| f.kind == FoodKind::Regular) {
                        self.spawn_food(FoodKind::Regular);
                    }
                    // Spawn bonus food every 10 points crossed
                    if self.score >= 10 && (self.score / 10) > ((self.score - pts) / 10) {
                        self.spawn_food(FoodKind::Bonus);
                    }
                }
                Mode::Maze => {
                    // Check if all food collected
                    if self.food.is_empty() {
                        // Advance level
                        self.level += 1;
                        self.generate_maze();
                        self.spawn_maze_food();
                        self.message = Some((format!("Level {}!", self.level), Instant::now()));
                    }
                }
                Mode::TimeAttack => {
                    // Respawn food
                    self.spawn_food(FoodKind::Regular);
                }
            }

            // Possibly spawn Golden food (10% chance)
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.1) && !self.food.iter().any(|f| f.kind == FoodKind::Golden) {
                self.spawn_food(FoodKind::Golden);
            }

            // Possibly spawn Shrink food (10% chance)
            if rng.gen_bool(0.1) && !self.food.iter().any(|f| f.kind == FoodKind::Shrink) {
                self.spawn_food(FoodKind::Shrink);
            }

            // Check win conditions
            match self.mode {
                Mode::Classic => {
                    if self.snake.len() >= (GRID_W * GRID_H) as usize {
                        self.status = Status::Won;
                        return;
                    }
                }
                Mode::Portal => {
                    if self.score >= self.portal_target {
                        self.status = Status::Won;
                        return;
                    }
                }
                _ => {}
            }
        } else {
            // No food eaten - remove tail normally
            self.snake.pop_back();
            self.idle_ticks += 1;
            self.streak = 0;

            // Insane difficulty: shrink on idle every 50 ticks
            if self.diff == Diff::Insane && self.idle_ticks > 0 && self.idle_ticks % 50 == 0 {
                if self.snake.len() > 3 {
                    self.snake.pop_back();
                }
            }
        }

        // Time Attack checks
        if self.mode == Mode::TimeAttack {
            // Check food timer expired
            if let Some(timer) = self.food_timer {
                if timer.elapsed().as_secs_f64() >= 8.0 {
                    if self.lives > 0 {
                        self.lives -= 1;
                    }
                    self.streak = 0;
                    // Remove current food and respawn
                    self.food.retain(|f| f.kind != FoodKind::Regular);
                    self.spawn_food(FoodKind::Regular);
                    self.food_timer = Some(Instant::now());

                    if self.lives == 0 {
                        self.status = Status::Over;
                        return;
                    }
                }
            }

            // Check total game timer
            if let Some(start) = self.game_timer_start {
                if start.elapsed().as_secs_f64() >= 60.0 {
                    self.status = Status::Won;
                    return;
                }
            }
        }

        // Expire message after 2 seconds
        if let Some((_, msg_time)) = &self.message {
            if msg_time.elapsed().as_secs_f64() > 2.0 {
                self.message = None;
            }
        }
    }
}

pub fn run(mode: Mode, diff: Diff) {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide
    )
    .unwrap();
    terminal::enable_raw_mode().unwrap();

    let result = game_loop(mode, diff, &mut stdout);

    terminal::disable_raw_mode().unwrap();
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen).unwrap();

    if let Some((score, m, d, level)) = result {
        if score > 0 {
            storage::record_score(m, d, score, level);
        }
    }
}

fn game_loop(mode: Mode, diff: Diff, stdout: &mut impl Write) -> Option<(u32, Mode, Diff, u32)> {
    let mut state = GameState::new(mode, diff);
    let mut last_tick = Instant::now();

    render::draw(stdout, &state);

    loop {
        let tick_ms = state.effective_tick_ms();
        let elapsed = last_tick.elapsed();
        let wait = Duration::from_millis(tick_ms).saturating_sub(elapsed);
        let poll_timeout = wait.min(Duration::from_millis(50));

        if event::poll(poll_timeout).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        state.handle_dir(Dir::Up)
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        state.handle_dir(Dir::Down)
                    }
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                        state.handle_dir(Dir::Left)
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                        state.handle_dir(Dir::Right)
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if state.status == Status::Running {
                            state.status = Status::Paused;
                        } else if state.status == Status::Paused {
                            state.status = Status::Running;
                            last_tick = Instant::now();
                        }
                        render::draw(stdout, &state);
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if matches!(state.status, Status::Over | Status::Won) {
                            let score = state.score;
                            let level = state.level;
                            storage::record_score(mode, diff, score, level);
                            state = GameState::new(mode, diff);
                            last_tick = Instant::now();
                            render::draw(stdout, &state);
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        let score = state.score;
                        let level = state.level;
                        return Some((score, mode, diff, level));
                    }
                    _ => {}
                }
            }
        }

        if state.status == Status::Running
            && last_tick.elapsed() >= Duration::from_millis(tick_ms)
        {
            state.tick();
            last_tick = Instant::now();
            render::draw(stdout, &state);
        } else if state.status == Status::Paused {
            // Redraw occasionally while paused
            render::draw(stdout, &state);
        } else if matches!(state.status, Status::Over | Status::Won) {
            render::draw(stdout, &state);
        }
    }
}
