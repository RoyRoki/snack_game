use std::time::Instant;

pub const GRID_W: i32 = 20;
pub const GRID_H: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn is_opposite(self, other: Dir) -> bool {
        matches!(
            (self, other),
            (Dir::Up, Dir::Down)
                | (Dir::Down, Dir::Up)
                | (Dir::Left, Dir::Right)
                | (Dir::Right, Dir::Left)
        )
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Classic,
    Portal,
    Maze,
    TimeAttack,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Classic => "Classic",
            Mode::Portal => "Portal",
            Mode::Maze => "Maze",
            Mode::TimeAttack => "Time Attack",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Mode::Classic => "Solid walls, speed increases, fill the grid to win",
            Mode::Portal => "Wrap-around walls, bonus foods, reach score target",
            Mode::Maze => "Navigate maze walls, collect all food to advance",
            Mode::TimeAttack => "8s per food, 3 lives, survive 60 seconds to win",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Diff {
    Nokiya,
    Easy,
    Normal,
    Hard,
    Insane,
}

impl Diff {
    pub fn tick_ms(self) -> u64 {
        match self {
            Diff::Nokiya => 200,
            Diff::Easy => 150,
            Diff::Normal => 100,
            Diff::Hard => 75,
            Diff::Insane => 50,
        }
    }

    pub fn multiplier(self) -> f64 {
        match self {
            Diff::Nokiya => 1.0,
            Diff::Easy => 1.5,
            Diff::Normal => 2.0,
            Diff::Hard => 3.0,
            Diff::Insane => 5.0,
        }
    }

    pub fn init_len(self) -> usize {
        match self {
            Diff::Nokiya => 3,
            Diff::Easy => 3,
            Diff::Normal => 5,
            Diff::Hard => 5,
            Diff::Insane => 7,
        }
    }

    pub fn obstacle_count(self) -> usize {
        match self {
            Diff::Nokiya => 0,
            Diff::Easy => 0,
            Diff::Normal => 2,
            Diff::Hard => 5,
            Diff::Insane => 10,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Diff::Nokiya => "Nokiya",
            Diff::Easy => "Easy",
            Diff::Normal => "Normal",
            Diff::Hard => "Hard",
            Diff::Insane => "Insane",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FoodKind {
    Regular,
    Bonus,
    Golden,
    Shrink,
}

impl FoodKind {
    pub fn base_pts(self) -> u32 {
        match self {
            FoodKind::Regular => 10,
            FoodKind::Bonus => 25,
            FoodKind::Golden => 50,
            FoodKind::Shrink => 15,
        }
    }

    pub fn grow(self) -> i32 {
        match self {
            FoodKind::Regular => 1,
            FoodKind::Bonus => 1,
            FoodKind::Golden => 2,
            FoodKind::Shrink => -2,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            FoodKind::Regular => "●",
            FoodKind::Bonus => "★",
            FoodKind::Golden => "◆",
            FoodKind::Shrink => "▼",
        }
    }

    pub fn expire_secs(self) -> Option<f64> {
        match self {
            FoodKind::Regular => None,
            FoodKind::Bonus => Some(5.0),
            FoodKind::Golden => Some(7.0),
            FoodKind::Shrink => Some(7.0),
        }
    }
}

pub struct Food {
    pub pos: Pos,
    pub kind: FoodKind,
    pub spawned: Instant,
}

impl Food {
    pub fn new(pos: Pos, kind: FoodKind) -> Self {
        Food {
            pos,
            kind,
            spawned: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expire_secs) = self.kind.expire_secs() {
            self.spawned.elapsed().as_secs_f64() >= expire_secs
        } else {
            false
        }
    }

    pub fn remaining_secs(&self) -> Option<f64> {
        if let Some(expire_secs) = self.kind.expire_secs() {
            let elapsed = self.spawned.elapsed().as_secs_f64();
            Some((expire_secs - elapsed).max(0.0))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Running,
    Paused,
    Over,
    Won,
}
