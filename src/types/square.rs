use std::fmt;
use std::ops;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

impl Square {
    pub const fn from_rank_file(rank: u8, file: u8) -> Self {
        debug_assert!(rank <= 7 && file <= 7);
        return Self::from_raw(rank * 8 + file);
    }

    pub const fn from_raw(value: u8) -> Self {
        debug_assert!(value <= Self::H8 as u8);
        unsafe { std::mem::transmute(value) }
    }

    pub const fn value(self) -> u8 {
        self as u8
    }

    pub const fn rank(self) -> u8 {
        self.value() / 8
    }

    pub const fn file(self) -> u8 {
        self.value() % 8
    }
}

impl ops::Add<i32> for Square {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        return Square::from_raw((self.value() as i32 + rhs) as u8);
    }
}

impl ops::AddAssign<i32> for Square {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

impl ops::Sub<i32> for Square {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        return Self::from_raw((self.value() as i32 - rhs) as u8);
    }
}

impl ops::SubAssign<i32> for Square {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl ops::Sub<Self> for Square {
    type Output = i32;
    fn sub(self, rhs: Self) -> Self::Output {
        self as i32 - rhs as i32
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub struct SquareParseErr;

impl FromStr for Square {
    type Err = SquareParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chrs = s.trim().chars();
        let Some(mut file) = chrs.next() else {
            return Err(SquareParseErr);
        };

        file = file.to_ascii_lowercase();

        let Some(rank) = chrs.next() else {
            return Err(SquareParseErr);
        };

        if file.is_ascii_alphabetic()
            && file <= 'h'
            && rank.is_ascii_digit()
            && rank >= '1'
            && rank <= '8'
        {
            return Ok(Square::from_rank_file(
                rank as u8 - '1' as u8,
                file as u8 - 'a' as u8,
            ));
        }

        Err(SquareParseErr)
    }
}
