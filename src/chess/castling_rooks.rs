use core::fmt;

use crate::types::{Color, Square};

#[derive(Clone, Copy, Debug)]
pub struct RookPair {
    pub king_side: Option<Square>,
    pub queen_side: Option<Square>,
}

impl RookPair {
    pub fn has_king_side(&self) -> bool {
        self.king_side.is_some()
    }

    pub fn has_queen_side(&self) -> bool {
        self.queen_side.is_some()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CastlingRooks {
    pub rooks: [RookPair; 2],
    pub frc: bool,
}

impl CastlingRooks {
    pub const DEFAULT: Self = Self {
        rooks: [
            RookPair {
                king_side: None,
                queen_side: None,
            },
            RookPair {
                king_side: None,
                queen_side: None,
            },
        ],
        frc: false,
    };

    pub const fn new(wk: Square, wq: Square, bk: Square, bq: Square, frc: bool) -> Self {
        Self {
            rooks: [
                RookPair {
                    king_side: Some(wk),
                    queen_side: Some(wq),
                },
                RookPair {
                    king_side: Some(bk),
                    queen_side: Some(bq),
                },
            ],
            frc: frc,
        }
    }

    pub const fn color(&self, c: Color) -> &RookPair {
        &self.rooks[c as usize]
    }

    pub fn color_mut(&mut self, c: Color) -> &mut RookPair {
        &mut self.rooks[c as usize]
    }
}

impl fmt::Display for CastlingRooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.frc {
            todo!()
        } else {
            if self.color(Color::White).king_side.is_some() {
                write!(f, "K")?;
            }
            if self.color(Color::White).queen_side.is_some() {
                write!(f, "Q")?;
            }
            if self.color(Color::Black).king_side.is_some() {
                write!(f, "k")?;
            }
            if self.color(Color::Black).queen_side.is_some() {
                write!(f, "q")?;
            }
        }
        Ok(())
    }
}
