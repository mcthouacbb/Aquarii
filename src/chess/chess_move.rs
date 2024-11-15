use std::{fmt, str::FromStr};

use crate::types::{Color, Piece, PieceType, Square};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveKind {
    None = 0,
    Enpassant,
    Castle,
    Promotion,
}

impl MoveKind {
    pub const fn from_raw(value: u8) -> Self {
        debug_assert!(value <= MoveKind::Promotion as u8);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    data: u16,
}

impl Move {
    pub const NULL: Move = Move { data: 0 };

    const fn new(from: Square, to: Square, kind: MoveKind, promo: u8) -> Self {
        Self {
            data: from as u16 | ((to as u16) << 6) | ((kind as u16) << 12) | ((promo as u16) << 14),
        }
    }

    pub const fn normal(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::None, 0)
    }

    pub const fn castle(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::Castle, 0)
    }

    pub const fn enpassant(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::Enpassant, 0)
    }

    pub const fn promo(from: Square, to: Square, promo: PieceType) -> Self {
        Self::new(
            from,
            to,
            MoveKind::Promotion,
            promo as u8 - PieceType::Knight as u8,
        )
    }

    pub const fn from_sq(&self) -> Square {
        Square::from_raw((self.data & 63) as u8)
    }

    pub const fn to_sq(&self) -> Square {
        Square::from_raw(((self.data >> 6) & 63) as u8)
    }

    pub const fn kind(&self) -> MoveKind {
        MoveKind::from_raw(((self.data >> 12) & 3) as u8)
    }

    pub const fn promo_piece(&self) -> PieceType {
        PieceType::from_raw(((self.data >> 14) + PieceType::Knight as u16) as u8)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from_sq(),
            self.to_sq(),
            if self.kind() == MoveKind::Promotion {
                Piece::new(Color::Black, self.promo_piece()).char_repr()
            } else {
                ' '
            }
        )
    }
}

pub struct MoveParseErr;

impl FromStr for Move {
    type Err = MoveParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from_str, rest) = s.split_at(2);
        let (to_str, promo) = rest.split_at(2);

        let from_sq = from_str.parse::<Square>().map_err(|_| MoveParseErr)?;
        let to_sq = to_str.parse::<Square>().map_err(|_| MoveParseErr)?;

        if promo.starts_with('n') || promo.starts_with('N') {
            return Ok(Move::promo(from_sq, to_sq, PieceType::Knight));
        } else if promo.starts_with('b') || promo.starts_with('B') {
            return Ok(Move::promo(from_sq, to_sq, PieceType::Bishop));
        } else if promo.starts_with('r') || promo.starts_with('R') {
            return Ok(Move::promo(from_sq, to_sq, PieceType::Rook));
        } else if promo.starts_with('q') || promo.starts_with('Q') {
            return Ok(Move::promo(from_sq, to_sq, PieceType::Queen));
        }

        Ok(Move::normal(from_sq, to_sq))
    }
}
