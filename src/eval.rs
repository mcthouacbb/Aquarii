use std::{
    fmt::Debug,
    ops::{self, Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, Board},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait EvalScoreType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<i32, Output = Self>
    + Div<i32, Output = Self>
{
}

impl EvalScoreType for i32 {}

pub trait EvalScorePairType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<i32, Output = Self>
{
    type ScoreType: EvalScoreType;

    fn mg(&self) -> Self::ScoreType;
    fn eg(&self) -> Self::ScoreType;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScorePair(i32);

impl ScorePair {
    pub const fn new(mg: i32, eg: i32) -> Self {
        Self((((eg as u32) << 16).wrapping_add(mg as u32)) as i32)
    }

    pub const fn mg(&self) -> i32 {
        self.0 as i16 as i32
    }

    pub const fn eg(&self) -> i32 {
        ((self.0.wrapping_add(0x8000)) as u32 >> 16) as i16 as i32
    }
}

impl ops::Add for ScorePair {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Sub for ScorePair {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::Mul<i32> for ScorePair {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Mul<ScorePair> for i32 {
    type Output = ScorePair;
    fn mul(self, rhs: ScorePair) -> Self::Output {
        ScorePair(self * rhs.0)
    }
}

impl ops::Neg for ScorePair {
    type Output = ScorePair;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl ops::AddAssign for ScorePair {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::SubAssign for ScorePair {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl ops::MulAssign<i32> for ScorePair {
    fn mul_assign(&mut self, rhs: i32) {
        self.0 *= rhs;
    }
}

impl EvalScorePairType for ScorePair {
    type ScoreType = i32;

    fn mg(&self) -> Self::ScoreType {
        self.mg()
    }

    fn eg(&self) -> Self::ScoreType {
        self.eg()
    }
}

#[allow(non_snake_case)]
const fn S(mg: i32, eg: i32) -> ScorePair {
    ScorePair::new(mg, eg)
}

pub trait EvalValues {
    type ScoreType: EvalScoreType;
    type ScorePairType: EvalScorePairType<ScoreType = Self::ScoreType>;

    fn material(pt: PieceType) -> Self::ScorePairType;
    fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType;
    fn mobility(pt: PieceType, mob: u32) -> Self::ScorePairType;
    fn passed_pawn(rank: u8) -> Self::ScorePairType;
    fn our_passer_dist(dist: i32) -> Self::ScorePairType;
    fn their_passer_dist(dist: i32) -> Self::ScorePairType;
    fn passed_blocked(rank: u8) -> Self::ScorePairType;
    fn passed_safe_adv(rank: u8) -> Self::ScorePairType;
    fn pawn_phalanx(rank: u8) -> Self::ScorePairType;
    fn defended_pawn(rank: u8) -> Self::ScorePairType;
    fn safe_knight_check() -> Self::ScorePairType;
    fn safe_bishop_check() -> Self::ScorePairType;
    fn safe_rook_check() -> Self::ScorePairType;
    fn safe_queen_check() -> Self::ScorePairType;
    fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType;
    fn king_attacks(attacks: u32) -> Self::ScorePairType;
    fn pawn_shield(edge_dist: u8, rank: u8) -> Self::ScorePairType;
    fn pawn_storm(edge_dist: u8, rank: u8) -> Self::ScorePairType;
    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn push_threat(stm: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(62,156), S(219,333), S(252,358), S(319,608), S(624,1138), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(79,87), S(43,97), S(32,85), S(64,64), S(50,82), S(-6,106), S(8,106), S(16,102),
        S(5,66), S(3,54), S(-1,41), S(33,8), S(21,14), S(-29,45), S(3,50), S(-0,60),
        S(-17,9), S(-1,-3), S(-2,-26), S(1,-33), S(10,-40), S(-1,-32), S(11,-20), S(-9,-8),
        S(-29,-20), S(-22,-22), S(-4,-48), S(-4,-46), S(-0,-55), S(-1,-51), S(-15,-32), S(-20,-36),
        S(-32,-29), S(-14,-33), S(-17,-48), S(-17,-36), S(-7,-44), S(-18,-43), S(14,-47), S(-21,-40),
        S(-35,-16), S(-8,-29), S(-19,-37), S(-25,-47), S(-15,-33), S(-4,-37), S(20,-47), S(-22,-36),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-51,-42), S(-8,-22), S(-40,1), S(-13,-2), S(39,-18), S(-66,-14), S(-37,-20), S(2,-72),
        S(-5,-5), S(-8,3), S(6,-1), S(9,-1), S(11,2), S(12,-5), S(-41,5), S(15,-6),
        S(16,-5), S(10,2), S(3,20), S(25,14), S(27,14), S(38,17), S(16,10), S(26,4),
        S(19,3), S(16,16), S(19,23), S(32,26), S(32,27), S(32,23), S(30,19), S(40,2),
        S(5,9), S(4,6), S(11,27), S(10,27), S(9,29), S(25,21), S(20,9), S(13,13),
        S(-19,-13), S(-4,-2), S(-10,10), S(-2,22), S(5,18), S(-4,9), S(12,-2), S(-8,-11),
        S(-18,-31), S(-22,1), S(-10,-14), S(-7,7), S(-2,-0), S(2,-9), S(-8,-3), S(-5,-12),
        S(-42,-38), S(-29,-26), S(-24,-13), S(-20,-8), S(-16,-4), S(-8,-8), S(-28,-7), S(-30,-24),
    ],
    [
        S(-11,-0), S(-35,4), S(-26,-8), S(-39,-4), S(-12,-10), S(-65,-2), S(-7,3), S(7,-5),
        S(-19,-4), S(-16,-1), S(-1,-2), S(-3,-3), S(-35,-1), S(-1,-2), S(-42,4), S(-20,-11),
        S(10,-6), S(16,1), S(5,16), S(19,7), S(30,1), S(21,24), S(14,9), S(30,-4),
        S(5,-10), S(8,14), S(8,21), S(24,26), S(22,28), S(18,20), S(18,8), S(14,-7),
        S(10,-15), S(-2,7), S(11,18), S(14,27), S(12,23), S(10,16), S(3,11), S(18,-19),
        S(4,-10), S(14,0), S(3,14), S(3,14), S(1,11), S(7,7), S(13,-1), S(7,-10),
        S(6,-8), S(8,-11), S(7,-13), S(-7,-9), S(-1,-8), S(13,-13), S(21,-9), S(3,-28),
        S(-16,-7), S(-10,-9), S(-6,-28), S(-28,-10), S(-13,-19), S(-18,-5), S(-9,-20), S(-15,-4),
    ],
    [
        S(45,9), S(60,7), S(51,11), S(64,1), S(62,4), S(72,-1), S(55,4), S(45,5),
        S(19,19), S(23,21), S(37,13), S(60,0), S(66,-4), S(61,-2), S(45,9), S(44,10),
        S(-15,17), S(2,12), S(-4,6), S(15,-8), S(26,-11), S(28,-8), S(20,-2), S(10,-1),
        S(-22,16), S(-13,12), S(-3,7), S(-2,-1), S(5,-9), S(6,-5), S(9,-0), S(2,-6),
        S(-39,15), S(-34,16), S(-19,10), S(-26,8), S(-20,2), S(-18,2), S(-5,0), S(-27,2),
        S(-48,13), S(-34,8), S(-37,8), S(-37,7), S(-27,-3), S(-28,-3), S(-11,-5), S(-34,1),
        S(-55,-1), S(-39,-10), S(-36,-5), S(-33,-10), S(-28,-14), S(-28,-15), S(-11,-27), S(-34,-26),
        S(-36,-13), S(-30,-4), S(-22,-1), S(-16,-4), S(-10,-13), S(-20,-9), S(-11,-21), S(-23,-32),
    ],
    [
        S(6,2), S(14,17), S(40,0), S(20,40), S(36,33), S(90,12), S(46,17), S(32,13),
        S(-5,9), S(-30,56), S(-11,67), S(-4,58), S(3,72), S(37,74), S(6,50), S(34,18),
        S(-10,-4), S(-10,32), S(-12,53), S(-5,59), S(-1,77), S(39,77), S(16,66), S(37,15),
        S(-13,3), S(-18,38), S(-20,48), S(-24,73), S(-19,77), S(-9,61), S(-6,51), S(42,-61),
        S(-7,-20), S(-23,15), S(-21,38), S(-24,54), S(-16,34), S(-11,34), S(-1,12), S(2,-18),
        S(-19,-35), S(-12,-12), S(-19,17), S(-16,-12), S(-14,-7), S(-5,-7), S(-0,-19), S(-0,-46),
        S(-17,-49), S(-15,-36), S(-4,-70), S(-10,-68), S(-5,-59), S(1,-83), S(6,-79), S(6,-82),
        S(-14,-72), S(-14,-83), S(-7,-83), S(-7,-62), S(-0,-86), S(-10,-118), S(-19,-105), S(-4,-99),
    ],
    [
        S(-85,34), S(82,-6), S(38,-1), S(121,-36), S(136,-61), S(126,-36), S(47,32), S(124,-3),
        S(-165,64), S(-21,64), S(73,7), S(72,7), S(46,15), S(83,10), S(109,27), S(-18,30),
        S(-54,38), S(7,53), S(7,32), S(31,24), S(40,24), S(86,24), S(55,48), S(-118,58),
        S(-116,22), S(-44,44), S(-4,23), S(-61,36), S(-51,35), S(-51,39), S(-80,57), S(-155,35),
        S(-141,7), S(9,8), S(-38,14), S(-60,22), S(-68,24), S(-46,20), S(-59,26), S(-130,10),
        S(1,-30), S(35,-11), S(-23,-0), S(-25,5), S(-34,6), S(-25,-0), S(12,-2), S(-33,-17),
        S(41,-51), S(43,-31), S(22,-35), S(3,-25), S(-7,-19), S(9,-30), S(49,-30), S(38,-46),
        S(-19,-85), S(54,-62), S(40,-66), S(-19,-60), S(25,-79), S(-6,-67), S(52,-52), S(38,-86),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-28,-54), S(-27,-41), S(-14,-13), S(-4,3), S(2,12), S(6,23), S(12,26), S(22,27), S(32,17), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(0,-53), S(-14,-49), S(-22,-14), S(-15,5), S(-8,11), S(-4,19), S(-3,22), S(0,20), S(3,19), S(6,14), S(5,14), S(13,-1), S(12,4), S(28,-10), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-59,-95), S(-9,-74), S(-15,-52), S(-16,-14), S(-12,0), S(-10,12), S(-6,19), S(-5,22), S(1,25), S(5,26), S(11,29), S(15,30), S(19,33), S(22,35), S(60,3), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-203,-339), S(-203,-339), S(-130,-332), S(-9,-110), S(-7,-13), S(-36,115), S(-13,62), S(-9,66), S(-6,70), S(-6,80), S(-6,93), S(-5,99), S(-2,101), S(-1,100), S(-1,99), S(3,97), S(4,94), S(8,85), S(11,73), S(15,67), S(25,49), S(48,15), S(58,9), S(86,-29), S(83,-40), S(123,-69), S(75,-43), S(98,-59)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(1,-77), S(-8,-57), S(-20,-7), S(-7,37), S(-1,84), S(87,196), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-11,57), S(-1,34), S(5,18), S(6,12), S(6,14), S(10,15), S(-1,13)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(5,-17), S(15,15), S(6,32), S(1,40), S(-4,43), S(-2,44), S(-17,41)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(6,-28), S(14,-57), S(22,-117), S(-42,-246)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-5,15), S(-14,44), S(-7,65), S(-17,137)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(5,-3), S(8,9), S(13,26), S(22,74), S(103,136), S(235,295), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(12,18), S(10,19), S(15,29), S(57,64), S(166,72), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(17,2);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(6,19);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(61,-2);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(27,16);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(12,14), S(9,17), S(11,11), S(0,65)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-25,32), S(-38,13), S(-39,8), S(-42,8), S(-33,-1), S(-19,-11), S(3,-20), S(33,-37), S(71,-53), S(106,-61), S(145,-84), S(186,-103), S(221,-135), S(245,-50)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(16,6), S(-5,14), S(-11,16), S(-2,8), S(12,-9), S(11,-26), S(-7,-41), S(0,0)],
    [S(7,7), S(-14,-2), S(-13,7), S(-9,5), S(7,-8), S(2,-25), S(13,-43), S(0,0)],
    [S(8,5), S(-6,0), S(3,-1), S(3,1), S(2,-4), S(-11,-8), S(-6,-31), S(0,0)],
    [S(5,5), S(-3,-2), S(-7,8), S(-6,9), S(-4,5), S(11,-12), S(55,-34), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(4,27), S(-58,-33), S(25,-53), S(8,-6), S(3,14), S(2,21), S(4,17), S(0,0)],
    [S(-2,11), S(-5,-57), S(46,-47), S(-7,-9), S(-1,-3), S(-12,7), S(-8,9), S(0,0)],
    [S(-4,11), S(51,-63), S(61,-49), S(4,-6), S(-3,7), S(-8,12), S(-3,8), S(0,0)],
    [S(2,5), S(38,-57), S(-0,-25), S(-0,-0), S(-0,7), S(-3,4), S(-6,10), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-23,-73), S(60,45), S(45,60), S(46,37), S(-23,198), S(0,0)],
    [S(-18,-53), S(120,211), S(134,243), S(132,486), S(332,1061), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,41), S(-26,-29), S(31,48), S(52,39), S(51,34), S(0,0)],
        [S(1,14), S(-29,-42), S(25,40), S(40,46), S(36,63), S(0,0)],
    ],
    [
        [S(14,57), S(12,60), S(81,171), S(75,397), S(245,1008), S(0,0)],
        [S(-7,12), S(-26,-35), S(35,47), S(60,193), S(217,672), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,31), S(43,58), S(-7,-4), S(45,59), S(70,95), S(0,0)],
        [S(1,8), S(23,39), S(-19,-34), S(36,79), S(49,141), S(0,0)],
    ],
    [
        [S(23,50), S(81,163), S(45,70), S(120,417), S(279,948), S(0,0)],
        [S(-5,7), S(18,45), S(-21,-26), S(55,231), S(294,511), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(4,58), S(36,68), S(51,76), S(-78,-175), S(77,56), S(0,0)],
        [S(-6,30), S(13,28), S(23,30), S(-91,-185), S(77,88), S(0,0)],
    ],
    [
        [S(4,79), S(47,209), S(84,231), S(-42,274), S(290,1155), S(0,0)],
        [S(-8,16), S(14,10), S(21,11), S(-81,-196), S(212,601), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(12,1), S(33,27), S(42,41), S(45,44), S(-84,-270), S(0,0)],
        [S(-2,2), S(3,0), S(7,6), S(2,13), S(-122,-256), S(0,0)],
    ],
    [
        [S(25,44), S(75,178), S(113,196), S(203,328), S(197,645), S(0,0)],
        [S(-5,2), S(-2,4), S(-0,19), S(-2,5), S(-120,-254), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(7,12), S(14,11)];
#[rustfmt::skip]
const TEMPO: i32 = 18;

pub struct EvalParams {}

impl EvalValues for EvalParams {
    type ScoreType = i32;
    type ScorePairType = ScorePair;

    fn material(pt: PieceType) -> Self::ScorePairType {
        MATERIAL[pt as usize]
    }

    fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType {
        PSQT[pt as usize][sq.relative_sq(c).flip().value() as usize]
    }

    fn mobility(pt: PieceType, mob: u32) -> Self::ScorePairType {
        MOBILITY[pt as usize - PieceType::Knight as usize][mob as usize]
    }

    fn passed_pawn(rank: u8) -> Self::ScorePairType {
        PASSED_PAWN[rank as usize]
    }

    fn our_passer_dist(dist: i32) -> Self::ScorePairType {
        OUR_PASSER_DIST[dist as usize]
    }

    fn their_passer_dist(dist: i32) -> Self::ScorePairType {
        THEIR_PASSER_DIST[dist as usize]
    }

    fn passed_blocked(rank: u8) -> Self::ScorePairType {
        PASSED_BLOCKED[(rank - 3) as usize]
    }

    fn passed_safe_adv(rank: u8) -> Self::ScorePairType {
        PASSED_SAFE_ADV[(rank - 3) as usize]
    }

    fn pawn_phalanx(rank: u8) -> Self::ScorePairType {
        PAWN_PHALANX[rank as usize]
    }

    fn defended_pawn(rank: u8) -> Self::ScorePairType {
        DEFENDED_PAWN[rank as usize]
    }

    fn safe_knight_check() -> Self::ScorePairType {
        SAFE_KNIGHT_CHECK
    }

    fn safe_bishop_check() -> Self::ScorePairType {
        SAFE_BISHOP_CHECK
    }

    fn safe_rook_check() -> Self::ScorePairType {
        SAFE_ROOK_CHECK
    }

    fn safe_queen_check() -> Self::ScorePairType {
        SAFE_QUEEN_CHECK
    }

    fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType {
        KING_ATTACKER_WEIGHT[pt as usize - PieceType::Knight as usize]
    }

    fn king_attacks(attacks: u32) -> Self::ScorePairType {
        KING_ATTACKS[attacks as usize]
    }

    fn pawn_shield(edge_dist: u8, rank: u8) -> Self::ScorePairType {
        PAWN_SHIELD[edge_dist as usize][rank as usize]
    }

    fn pawn_storm(edge_dist: u8, rank: u8) -> Self::ScorePairType {
        PAWN_STORM[edge_dist as usize][rank as usize]
    }

    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType {
        THREAT_BY_PAWN[stm as usize][pt as usize]
    }

    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_KNIGHT[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_BISHOP[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_ROOK[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_QUEEN[stm as usize][defended as usize][pt as usize]
    }

    fn push_threat(stm: bool) -> Self::ScorePairType {
        PUSH_THREAT[stm as usize]
    }

    fn tempo() -> Self::ScoreType {
        TEMPO
    }
}

struct EvalData<ScorePairType: EvalScorePairType> {
    attacked: [Bitboard; 2],
    attacked_by: [[Bitboard; 6]; 2],
    attacked_by_2: [Bitboard; 2],
    king_ring: [Bitboard; 2],
    king_attack_weight: [ScorePairType; 2],
    king_attacks: [i32; 2],
}

impl<ScorePairType: EvalScorePairType> Default for EvalData<ScorePairType> {
    fn default() -> Self {
        Self {
            attacked: [Bitboard::NONE; 2],
            attacked_by: [[Bitboard::NONE; 6]; 2],
            attacked_by_2: [Bitboard::NONE; 2],
            king_ring: [Bitboard::NONE; 2],
            king_attack_weight: [ScorePairType::default(), ScorePairType::default()],
            king_attacks: [0; 2],
        }
    }
}

fn evaluate_piece<Params: EvalValues>(
    board: &Board,
    pt: PieceType,
    color: Color,
    eval_data: &mut EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let opp_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));
    let mobility_area = !attacks::pawn_attacks_bb(!color, opp_pawns);

    let mut pieces = board.colored_pieces(Piece::new(color, pt));

    while pieces.any() {
        let sq = pieces.poplsb();

        let attacks = attacks::piece_attacks(pt, sq, board.occ());
        let mobility = (attacks & mobility_area).popcount();
        eval += Params::mobility(pt, mobility);

        eval_data.attacked_by_2[color as usize] |= attacks & eval_data.attacked[color as usize];
        eval_data.attacked[color as usize] |= attacks;
        eval_data.attacked_by[color as usize][pt as usize] |= attacks;

        let king_ring_attacks = eval_data.king_ring[!color as usize] & attacks;
        if king_ring_attacks.any() {
            eval_data.king_attack_weight[color as usize] += Params::king_attacker_weight(pt);
            eval_data.king_attacks[color as usize] += king_ring_attacks.popcount() as i32;
        }
    }
    eval
}

fn evaluate_king_pawn_file<Params: EvalValues>(
    board: &Board,
    color: Color,
    their_king: Square,
    file: u8,
) -> Params::ScorePairType {
    let edge_dist = file.min(7 - file);

    let their_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));
    let file_pawns = their_pawns & Bitboard::file(file);
    let shield_rank = if file_pawns.any() {
        if color == Color::White {
            file_pawns.msb()
        } else {
            file_pawns.lsb()
        }
        .relative_sq(!color)
        .rank()
    } else {
        0
    };

    let our_pawns = board.colored_pieces(Piece::new(color, PieceType::Pawn));
    let file_pawns = our_pawns & Bitboard::file(file);
    let storm_rank = if file_pawns.any() {
        if color == Color::White {
            file_pawns.msb()
        } else {
            file_pawns.lsb()
        }
        .relative_sq(!color)
        .rank()
    } else {
        0
    };
    return Params::pawn_shield(edge_dist, shield_rank) + Params::pawn_storm(edge_dist, storm_rank);
}

fn evaluate_kings<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let their_king = board.king_sq(!color);

    let middle_file = their_king.file().clamp(1, 6);
    for file in (middle_file - 1)..=(middle_file + 1) {
        eval += evaluate_king_pawn_file::<Params>(board, color, their_king, file);
    }

    let rook_check_squares = attacks::rook_attacks(their_king, board.occ());
    let bishop_check_squares = attacks::bishop_attacks(their_king, board.occ());

    let knight_checks = eval_data.attacked_by[color as usize][PieceType::Knight as usize]
        & attacks::knight_attacks(their_king);
    let bishop_checks =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & bishop_check_squares;
    let rook_checks =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & rook_check_squares;
    let queen_checks = eval_data.attacked_by[color as usize][PieceType::Queen as usize]
        & (bishop_check_squares | rook_check_squares);

    let weak = !eval_data.attacked[!color as usize]
        | (!eval_data.attacked_by_2[!color as usize]
            & eval_data.attacked_by[!color as usize][PieceType::King as usize]);
    let safe = !board.colors(color)
        & (!eval_data.attacked[!color as usize] | (weak & eval_data.attacked_by_2[color as usize]));

    eval += Params::safe_knight_check() * (knight_checks & safe).popcount() as i32;
    eval += Params::safe_bishop_check() * (bishop_checks & safe).popcount() as i32;
    eval += Params::safe_rook_check() * (rook_checks & safe).popcount() as i32;
    eval += Params::safe_queen_check() * (queen_checks & safe).popcount() as i32;

    eval += eval_data.king_attack_weight[color as usize].clone();
    eval += Params::king_attacks(eval_data.king_attacks[color as usize].min(13) as u32);

    return eval;
}

fn evaluate_threats<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let third_rank = if color == Color::White {
        Bitboard::RANK_3
    } else {
        Bitboard::RANK_6
    };

    let stm = color == board.stm();
    let mut eval = Params::ScorePairType::default();

    let defended_bb = eval_data.attacked_by_2[!color as usize]
        | eval_data.attacked_by[!color as usize][PieceType::Pawn as usize]
        | (eval_data.attacked[!color as usize] & !eval_data.attacked_by_2[color as usize]);

    let mut pawn_threats =
        eval_data.attacked_by[color as usize][PieceType::Pawn as usize] & board.colors(!color);
    while pawn_threats.any() {
        let threatened = board.piece_at(pawn_threats.poplsb()).unwrap().piece_type();
        eval += Params::threat_by_pawn(stm, threatened);
    }

    let mut knight_threats =
        eval_data.attacked_by[color as usize][PieceType::Knight as usize] & board.colors(!color);
    while knight_threats.any() {
        let threat = knight_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_knight(stm, threatened, defended);
    }

    let mut bishop_threats =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & board.colors(!color);
    while bishop_threats.any() {
        let threat = bishop_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_bishop(stm, threatened, defended);
    }

    let mut rook_threats =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & board.colors(!color);
    while rook_threats.any() {
        let threat = rook_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_rook(stm, threatened, defended);
    }

    let mut queen_threats =
        eval_data.attacked_by[color as usize][PieceType::Queen as usize] & board.colors(!color);
    while queen_threats.any() {
        let threat = queen_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_queen(stm, threatened, defended);
    }

    let non_pawns = board.colors(!color) & !board.pieces(PieceType::Pawn);
    let mut pushes = attacks::pawn_pushes_bb(
        color,
        board.colored_pieces(Piece::new(color, PieceType::Pawn)),
    ) & !board.occ();
    pushes |= attacks::pawn_pushes_bb(color, pushes & third_rank) & !board.occ();

    let push_threats = attacks::pawn_attacks_bb(color, pushes) & non_pawns;
    eval += Params::push_threat(stm) * push_threats.popcount() as i32;

    eval
}

fn evaluate_pawns<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    const RANK_4: u8 = 3;

    let mut eval = Params::ScorePairType::default();
    let our_pawns = board.colored_pieces(Piece::new(color, PieceType::Pawn));
    let their_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));

    let mut tmp = our_pawns;
    while tmp.any() {
        let sq = tmp.poplsb();
        let push_sq = if color == Color::White {
            sq + 8
        } else {
            sq - 8
        };
        let relative_rank = sq.relative_sq(color).rank();
        let stoppers = their_pawns & attacks::passed_pawn_span(color, sq);
        if stoppers.empty() {
            eval += Params::passed_pawn(relative_rank);
            let our_passer_dist = Square::chebyshev(board.king_sq(color), sq);
            let their_passer_dist = Square::chebyshev(board.king_sq(!color), sq);
            eval += Params::our_passer_dist(our_passer_dist)
                + Params::their_passer_dist(their_passer_dist);

            if relative_rank >= RANK_4 {
                if board.occ().has(push_sq) {
                    eval += Params::passed_blocked(relative_rank);
                }

                if !eval_data.attacked[!color as usize].has(push_sq) {
                    eval += Params::passed_safe_adv(relative_rank);
                }
            }
        }
    }

    let mut phalanxes = our_pawns & our_pawns.west();
    while phalanxes.any() {
        eval += Params::pawn_phalanx(phalanxes.poplsb().relative_sq(color).rank());
    }
    let mut defended = our_pawns & attacks::pawn_attacks_bb(color, our_pawns);
    while defended.any() {
        eval += Params::defended_pawn(defended.poplsb().relative_sq(color).rank());
    }
    eval
}

pub fn eval_impl<Params: EvalValues>(board: &Board) -> Params::ScoreType {
    let stm = board.stm();
    let mut eval = Params::ScorePairType::default();
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ] {
        let mut stm_bb = board.colored_pieces(Piece::new(stm, pt));
        let mut nstm_bb = board.colored_pieces(Piece::new(!stm, pt));

        while stm_bb.any() {
            eval += Params::material(pt) + Params::psqt(stm, pt, stm_bb.poplsb());
        }

        while nstm_bb.any() {
            eval -= Params::material(pt) + Params::psqt(!stm, pt, nstm_bb.poplsb());
        }
    }

    let mut eval_data = EvalData::default();
    // TODO: handle pawn attacks
    let wking_atks = attacks::king_attacks(board.king_sq(Color::White));
    let bking_atks = attacks::king_attacks(board.king_sq(Color::Black));
    eval_data.attacked[Color::White as usize] = wking_atks;
    eval_data.attacked[Color::Black as usize] = bking_atks;
    eval_data.attacked_by[Color::White as usize][PieceType::King as usize] = wking_atks;
    eval_data.attacked_by[Color::Black as usize][PieceType::King as usize] = bking_atks;
    eval_data.attacked_by[Color::White as usize][PieceType::Pawn as usize] =
        attacks::pawn_attacks_bb(Color::White, board.colored_pieces(Piece::WhitePawn));
    eval_data.attacked_by[Color::Black as usize][PieceType::Pawn as usize] =
        attacks::pawn_attacks_bb(Color::Black, board.colored_pieces(Piece::BlackPawn));

    eval_data.king_ring[Color::White as usize] =
        (wking_atks | wking_atks.north()) & !Bitboard::from_square(board.king_sq(Color::White));
    eval_data.king_ring[Color::Black as usize] =
        (bking_atks | bking_atks.south()) & !Bitboard::from_square(board.king_sq(Color::Black));

    eval += evaluate_piece::<Params>(board, PieceType::Knight, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Knight, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Bishop, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Bishop, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Rook, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Rook, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Queen, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Queen, !stm, &mut eval_data);

    eval += evaluate_kings::<Params>(board, stm, &eval_data)
        - evaluate_kings::<Params>(board, !stm, &eval_data);
    eval += evaluate_threats::<Params>(board, stm, &eval_data)
        - evaluate_threats::<Params>(board, !stm, &eval_data);

    eval += evaluate_pawns::<Params>(board, stm, &eval_data)
        - evaluate_pawns::<Params>(board, !stm, &eval_data);

    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount()) as i32;

    (eval.mg() * phase.min(24) + eval.eg() * (24 - phase.min(24))) / 24 + Params::tempo()
}

pub fn eval(board: &Board) -> i32 {
    eval_impl::<EvalParams>(board)
}
