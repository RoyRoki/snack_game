use wasm_bindgen::prelude::*;
use crate::game::GameState;
use crate::types::*;

#[wasm_bindgen]
pub struct WasmGame {
    state: GameState,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(mode: u8, diff: u8) -> WasmGame {
        WasmGame { state: GameState::new(mode_of(mode), diff_of(diff)) }
    }

    pub fn tick(&mut self) { self.state.tick(); }

    pub fn input(&mut self, dir: u8) {
        let d = match dir { 0=>Dir::Up, 1=>Dir::Down, 2=>Dir::Left, 3=>Dir::Right, _=>return };
        self.state.handle_dir(d);
    }

    pub fn toggle_pause(&mut self) {
        self.state.status = match self.state.status {
            Status::Running => Status::Paused,
            Status::Paused  => Status::Running,
            s => s,
        };
    }

    pub fn restart(&mut self) {
        self.state = GameState::new(self.state.mode, self.state.diff);
    }

    pub fn save_score(&self, name: &str) {
        crate::storage::record_score_named(
            self.state.mode, self.state.diff,
            self.state.score, self.state.level, name,
        );
        crate::storage::update_profile_stats(self.state.foods_eaten as u64, self.state.max_streak);
    }

    pub fn high_scores(&self) -> String {
        let v = crate::storage::get_top_scores(self.state.mode, self.state.diff);
        let parts: Vec<String> = v.iter().map(|e| {
            format!(r#"{{"n":"{}","s":{},"l":{}}}"#,
                e.name.replace('"', "'"), e.score, e.level)
        }).collect();
        format!("[{}]", parts.join(","))
    }

    /// Returns a flat 400-byte grid: 20×20 cells, row-major.
    /// Values: 0=empty 1=body 2=head 3=regular 4=bonus 5=golden 6=shrink 7=mystery 8=wall
    pub fn grid(&self) -> Vec<u8> {
        let mut g = vec![0u8; (GRID_W * GRID_H) as usize];
        for p in &self.state.walls {
            g[(p.y * GRID_W + p.x) as usize] = 8;
        }
        for (i, p) in self.state.snake.iter().enumerate() {
            g[(p.y * GRID_W + p.x) as usize] = if i == 0 { 2 } else { 1 };
        }
        for f in &self.state.food {
            g[(f.pos.y * GRID_W + f.pos.x) as usize] = match f.kind {
                FoodKind::Regular => 3,
                FoodKind::Bonus   => 4,
                FoodKind::Golden  => 5,
                FoodKind::Shrink  => 6,
                FoodKind::Mystery => 7,
            };
        }
        g
    }

    pub fn score(&self)           -> u32  { self.state.score }
    pub fn lives(&self)           -> u8   { self.state.lives }
    pub fn level(&self)           -> u32  { self.state.level }
    pub fn streak(&self)          -> u32  { self.state.streak }
    pub fn tick_ms(&self)         -> u32  { self.state.effective_tick_ms() as u32 }
    pub fn status(&self)          -> u8   {
        match self.state.status {
            Status::Running => 0, Status::Paused => 1,
            Status::Over    => 2, Status::Won    => 3,
        }
    }
    pub fn danger(&self)          -> bool { self.state.danger_next_tick }
    pub fn mode_id(&self)         -> u8   {
        match self.state.mode {
            Mode::Classic => 0, Mode::Portal => 1,
            Mode::Maze    => 2, Mode::TimeAttack => 3,
        }
    }
    pub fn portal_target(&self)   -> u32  { self.state.portal_target }
    pub fn personal_best(&self)   -> u32  { self.state.personal_best }
    pub fn time_left(&self)       -> f32  { self.state.time_attack_remaining_secs() as f32 }
    pub fn food_timer_secs(&self) -> f32  { self.state.food_timer_remaining_secs() as f32 }
    pub fn daily_desc(&self)      -> String { self.state.daily_desc.clone() }
    pub fn daily_progress(&self)  -> u32  { self.state.daily_progress }
    pub fn daily_target(&self)    -> u32  { self.state.daily_target }
    pub fn daily_done(&self)      -> bool { self.state.daily_completed }
    pub fn message(&self)         -> String {
        self.state.message.as_ref().map(|(m, _)| m.clone()).unwrap_or_default()
    }
    pub fn achievement(&self)     -> String {
        self.state.pending_achievement.as_ref().map(|(m, _)| m.clone()).unwrap_or_default()
    }
    pub fn reversed(&self)        -> bool {
        self.state.reversed_until
            .map(|t| instant::Instant::now() < t)
            .unwrap_or(false)
    }
}

fn mode_of(v: u8) -> Mode {
    match v { 1=>Mode::Portal, 2=>Mode::Maze, 3=>Mode::TimeAttack, _=>Mode::Classic }
}
fn diff_of(v: u8) -> Diff {
    match v { 1=>Diff::Easy, 2=>Diff::Normal, 3=>Diff::Hard, 4=>Diff::Insane, _=>Diff::Nokiya }
}
