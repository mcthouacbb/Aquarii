use core::fmt;

pub fn sigmoid(x: f32, scale: f32) -> f32 {
    1.0 / (1.0 + (-x / scale).exp())
}

pub fn sigmoid_inv(x: f32, scale: f32) -> f32 {
    scale * (x / (1.0 - x)).ln()
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum GameResult {
    NonTerminal,
    Mated,
    Drawn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MateScore {
    Loss(u16),
    Win(u16),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Score {
    Win(u16),
    Draw,
    Loss(u16),
    Normal(f32),
}

impl Score {
    pub fn flip(&self) -> Self {
        match self {
            Self::Win(dist) => Self::Loss(*dist),
            Self::Draw => Self::Draw,
            Self::Loss(dist) => Self::Win(*dist),
            Self::Normal(score) => Self::Normal(1.0 - score),
        }
    }

    pub fn uci_str(&self) -> String {
        match self {
            Self::Win(dist) => format!("mate {}", (*dist + 1) / 2),
            Self::Draw => format!("cp 0"),
            Self::Loss(dist) => format!("mate -{}", *dist / 2),
            Self::Normal(score) => format!("cp {}", sigmoid_inv(*score, 400.0).round()),
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Win(dist) => write!(f, "win {} plies", *dist),
            Self::Draw => write!(f, "draw"),
            Self::Loss(dist) => write!(f, "loss {} plies", *dist),
            Self::Normal(score) => write!(f, "cp {}", sigmoid_inv(*score, 400.0).round()),
        }
    }
}
