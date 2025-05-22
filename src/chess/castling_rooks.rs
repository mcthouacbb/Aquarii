use core::fmt;

use crate::types::{Color, Square};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RookPair {
    pub king_side: Option<Square>,
    pub queen_side: Option<Square>,
}

impl RookPair {
    pub fn remove(&mut self, sq: Square) {
        if self.king_side == Some(sq) {
            self.king_side = None;
        } else if self.queen_side == Some(sq) {
            self.queen_side = None;
        }
    }

    pub fn remove_both(&mut self) {
        self.king_side = None;
        self.queen_side = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn right_bits(&self) -> u32 {
        let mut rights = 0;
        if self.color(Color::White).king_side.is_some() {
            rights |= 1;
        }
        if self.color(Color::White).queen_side.is_some() {
            rights |= 2;
        }
        if self.color(Color::Black).king_side.is_some() {
            rights |= 4;
        }
        if self.color(Color::Black).queen_side.is_some() {
            rights |= 8;
        }
        rights
    }

    pub const fn king_to(king_side: bool, c: Color) -> Square {
        [[Square::C1, Square::C8], [Square::G1, Square::G8]][king_side as usize][c as usize]
    }

    pub const fn rook_to(king_side: bool, c: Color) -> Square {
        [[Square::D1, Square::D8], [Square::F1, Square::F8]][king_side as usize][c as usize]
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
