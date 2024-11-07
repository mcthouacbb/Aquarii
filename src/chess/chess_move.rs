use crate::types::{PieceType, Square};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveKind {
    None,
    Enpassant,
    Castle,
    Promotion,
}

impl MoveKind {
    pub const fn from_raw(value: u8) -> Self {
        assert!(value <= MoveKind::Promotion as u8);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    data: u16,
}

impl Move {
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
