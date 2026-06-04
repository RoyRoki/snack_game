use instant::Instant;

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
    Mystery,
}

impl FoodKind {
    pub fn base_pts(self) -> u32 {
        match self {
            FoodKind::Regular => 10,
            FoodKind::Bonus => 25,
            FoodKind::Golden => 50,
            FoodKind::Shrink => 15,
            FoodKind::Mystery => 0,
        }
    }

    pub fn grow(self) -> i32 {
        match self {
            FoodKind::Regular => 1,
            FoodKind::Bonus => 1,
            FoodKind::Golden => 2,
            FoodKind::Shrink => -2,
            FoodKind::Mystery => 1,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            FoodKind::Regular => "●",
            FoodKind::Bonus => "★",
            FoodKind::Golden => "◆",
            FoodKind::Shrink => "▼",
            FoodKind::Mystery => "?",
        }
    }

    pub fn expire_secs(self) -> Option<f64> {
        match self {
            FoodKind::Regular => None,
            FoodKind::Bonus => Some(5.0),
            FoodKind::Golden => Some(7.0),
            FoodKind::Shrink => Some(7.0),
            FoodKind::Mystery => Some(5.0),
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

    #[allow(dead_code)]
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MysteryEffect {
    Jackpot,    // +50 pts  "JACKPOT!"
    Bonus,      // +30 pts  "Bonus!"
    SpeedBoost, // +20 pts + speed x2 for 5s  "TURBO!"
    Reverse,    // +15 pts + controls flip 3s  "Reversed!"
    Shrink,     // +10 pts + pop 3 extra tail cells  "Oops!"
}

impl MysteryEffect {
    pub fn pts(self) -> u32 {
        match self { Self::Jackpot => 50, Self::Bonus => 30, Self::SpeedBoost => 20, Self::Reverse => 15, Self::Shrink => 10 }
    }
    pub fn label(self) -> &'static str {
        match self { Self::Jackpot => "JACKPOT!", Self::Bonus => "Bonus!", Self::SpeedBoost => "TURBO!", Self::Reverse => "Reversed!", Self::Shrink => "Oops!" }
    }
    pub fn random() -> Self {
        use rand::Rng;
        match rand::thread_rng().gen_range(0u32..100) {
            0..=14  => Self::Jackpot,
            15..=34 => Self::Bonus,
            35..=54 => Self::SpeedBoost,
            55..=74 => Self::Reverse,
            _       => Self::Shrink,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Achievement {
    FirstBite, OnFire, SpeedDemon, TimeSurvivor, MazeRunner,
    PortalMaster, ClassicKing, CenturyClub, HighRoller, Collector,
}

impl Achievement {
    pub const ALL: &'static [Achievement] = &[
        Self::FirstBite, Self::OnFire, Self::SpeedDemon, Self::TimeSurvivor,
        Self::MazeRunner, Self::PortalMaster, Self::ClassicKing,
        Self::CenturyClub, Self::HighRoller, Self::Collector,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::FirstBite    => "first_bite",
            Self::OnFire       => "on_fire",
            Self::SpeedDemon   => "speed_demon",
            Self::TimeSurvivor => "time_survivor",
            Self::MazeRunner   => "maze_runner",
            Self::PortalMaster => "portal_master",
            Self::ClassicKing  => "classic_king",
            Self::CenturyClub  => "century_club",
            Self::HighRoller   => "high_roller",
            Self::Collector    => "collector",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::FirstBite    => "First Bite",
            Self::OnFire       => "On Fire",
            Self::SpeedDemon   => "Speed Demon",
            Self::TimeSurvivor => "Time Survivor",
            Self::MazeRunner   => "Maze Runner",
            Self::PortalMaster => "Portal Master",
            Self::ClassicKing  => "Classic King",
            Self::CenturyClub  => "Century Club",
            Self::HighRoller   => "High Roller",
            Self::Collector    => "Collector",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::FirstBite    => "Eat your first food",
            Self::OnFire       => "Reach a streak of 10",
            Self::SpeedDemon   => "Eat 5 foods on Insane difficulty",
            Self::TimeSurvivor => "Survive 60 seconds in Time Attack",
            Self::MazeRunner   => "Complete the Maze",
            Self::PortalMaster => "Win Portal mode",
            Self::ClassicKing  => "Fill the grid in Classic",
            Self::CenturyClub  => "Score 100+ in one game",
            Self::HighRoller   => "Score 500+ in one game",
            Self::Collector    => "Unlock all other achievements",
        }
    }
}
