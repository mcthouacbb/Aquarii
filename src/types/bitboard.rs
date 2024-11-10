use core::fmt;
use std::ops;

use super::Square;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const FILE_A: Self = Self(0x0101010101010101u64);
    pub const FILE_B: Self = Self(0x0202020202020202u64);
    pub const FILE_C: Self = Self(0x0404040404040404u64);
    pub const FILE_D: Self = Self(0x0808080808080808u64);
    pub const FILE_E: Self = Self(0x1010101010101010u64);
    pub const FILE_F: Self = Self(0x2020202020202020u64);
    pub const FILE_G: Self = Self(0x4040404040404040u64);
    pub const FILE_H: Self = Self(0x8080808080808080u64);

    pub const RANK_1: Self = Self(0x00000000000000FFu64);
    pub const RANK_2: Self = Self(0x000000000000FF00u64);
    pub const RANK_3: Self = Self(0x0000000000FF0000u64);
    pub const RANK_4: Self = Self(0x00000000FF000000u64);
    pub const RANK_5: Self = Self(0x000000FF00000000u64);
    pub const RANK_6: Self = Self(0x0000FF0000000000u64);
    pub const RANK_7: Self = Self(0x00FF000000000000u64);
    pub const RANK_8: Self = Self(0xFF00000000000000u64);

    pub const NONE: Self = Self(0u64);
    pub const ALL: Self = Self(!0u64);

    pub const fn file(file: u8) -> Bitboard {
        Self(Self::FILE_A.value() << file)
    }

    pub const fn from_square(sq: Square) -> Self {
        return Self(1 << sq.value());
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn north(self) -> Self {
        Self(self.value() << 8)
    }

    pub const fn south(self) -> Self {
        Self(self.value() >> 8)
    }

    pub const fn west(self) -> Self {
        Self((self.value() >> 1) & !Self::FILE_H.value())
    }

    pub const fn east(self) -> Self {
        Self((self.value() << 1) & !Self::FILE_A.value())
    }

    pub const fn north_west(self) -> Self {
        self.north().west()
    }

    pub const fn north_east(self) -> Self {
        self.north().east()
    }

    pub const fn south_west(self) -> Self {
        self.south().west()
    }

    pub const fn south_east(self) -> Self {
        self.south().east()
    }

    pub const fn lsb(self) -> Square {
        Square::from_raw(self.0.trailing_zeros() as u8)
    }

    pub const fn msb(self) -> Square {
        Square::from_raw((63 - self.0.leading_zeros()) as u8)
    }

    pub const fn popcount(self) -> u32 {
        self.value().count_ones()
    }

    pub fn poplsb(&mut self) -> Square {
        let lsb = self.lsb();
        self.0 &= self.0 - 1;
        lsb
    }

    pub const fn any(self) -> bool {
        self.value() > 0
    }

    pub const fn empty(self) -> bool {
        self.value() == 0
    }

    pub const fn multiple(self) -> bool {
        self.any() && (self.value() & (self.value() - 1)) > 0
    }

    pub const fn one(self) -> bool {
        self.any() && !self.multiple()
    }

    pub const fn has(self, sq: Square) -> bool {
        return ((self.value() >> (sq as u8)) & 1u64) > 0;
    }

    pub const fn bit_and(self, rhs: Self) -> Self {
        Self(self.value() & rhs.value())
    }

    pub const fn bit_or(self, rhs: Self) -> Self {
        Self(self.value() | rhs.value())
    }

    pub const fn bit_xor(self, rhs: Self) -> Self {
        Self(self.value() ^ rhs.value())
    }

    pub const fn bit_not(self) -> Self {
        Self(!self.value())
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.bit_and(rhs)
    }
}

impl ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bit_and(rhs);
    }
}

impl ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.bit_or(rhs)
    }
}

impl ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bit_or(rhs);
    }
}

impl ops::BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bit_xor(rhs)
    }
}

impl ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.bit_xor(rhs)
    }
}

impl ops::Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        self.bit_not()
    }
}

fn reverse(mut n: u8) -> u8 {
    n = (n & 0xF0) >> 4 | (n & 0x0F) << 4;
    n = (n & 0xCC) >> 2 | (n & 0x33) << 2;
    n = (n & 0xAA) >> 1 | (n & 0x55) << 1;
    return n;
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // this is cursed
        let mut bb = self.value();
        for _ in 0..8 {
            let row: u64 = bb >> 56;
            write!(f, "{:08b}\n", reverse(row as u8))?;
            bb <<= 8;
        }
        Ok(())
    }
}
