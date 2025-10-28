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
const MATERIAL: [ScorePair; 6] = [S(72,164), S(341,389), S(417,417), S(509,800), S(889,1500), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(95,41), S(-13,91), S(23,57), S(58,60), S(68,55), S(-15,69), S(-23,72), S(81,40),
        S(8,27), S(3,15), S(-4,12), S(42,-5), S(52,-1), S(-27,21), S(-6,29), S(-2,27),
        S(-10,11), S(-5,-6), S(-3,-24), S(5,-40), S(29,-39), S(-11,-24), S(-10,-12), S(-5,-3),
        S(-22,-7), S(-24,-7), S(-6,-35), S(-3,-43), S(6,-42), S(-11,-35), S(-16,-16), S(-26,-13),
        S(-38,-7), S(-30,-18), S(-18,-29), S(-21,-34), S(-4,-30), S(-16,-29), S(-2,-32), S(-44,-10),
        S(-24,2), S(-9,-5), S(-3,-16), S(-6,-29), S(4,-14), S(1,-15), S(12,-14), S(-31,3),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-51,-70), S(86,-90), S(27,-95), S(-28,-5), S(16,4), S(17,-162), S(108,-91), S(18,-146),
        S(9,-42), S(-38,8), S(14,16), S(19,-14), S(22,-15), S(-4,-20), S(-3,-12), S(59,-69),
        S(-10,-9), S(49,-47), S(18,32), S(15,42), S(34,29), S(76,21), S(52,-45), S(39,-16),
        S(12,12), S(6,18), S(9,45), S(34,45), S(11,59), S(12,63), S(27,46), S(46,19),
        S(-3,-6), S(2,10), S(2,42), S(5,46), S(-6,53), S(14,45), S(12,25), S(23,7),
        S(-40,-11), S(-33,12), S(-29,32), S(-18,41), S(-4,44), S(-21,30), S(-8,14), S(-23,12),
        S(-40,-37), S(-49,60), S(-30,-9), S(-26,20), S(-27,20), S(-29,16), S(-7,49), S(0,6),
        S(-113,-47), S(-54,6), S(-36,-12), S(-23,15), S(-14,24), S(-27,22), S(-40,-4), S(-60,-34),
    ],
    [
        S(-13,-40), S(38,-29), S(8,-45), S(-46,-2), S(-23,-20), S(-56,-44), S(176,-73), S(-22,-52),
        S(8,-45), S(-35,11), S(-2,7), S(33,-36), S(-19,-20), S(-24,2), S(-139,16), S(1,-89),
        S(13,1), S(18,11), S(-1,14), S(10,14), S(10,10), S(40,19), S(33,20), S(41,14),
        S(-25,8), S(-7,28), S(-4,17), S(21,30), S(6,29), S(14,24), S(-5,31), S(-7,-1),
        S(2,-22), S(-25,14), S(9,22), S(-5,37), S(-1,31), S(-4,27), S(5,6), S(8,-13),
        S(-13,-12), S(7,12), S(-3,24), S(-3,20), S(-9,25), S(-2,9), S(12,0), S(7,-27),
        S(21,8), S(0,-9), S(4,-1), S(-26,12), S(-18,17), S(-2,8), S(10,5), S(17,-25),
        S(-1,7), S(24,-6), S(-26,8), S(-16,3), S(-12,13), S(-30,28), S(11,-11), S(18,-19),
    ],
    [
        S(-2,13), S(20,39), S(-15,59), S(41,12), S(31,15), S(16,38), S(75,11), S(78,-20),
        S(12,15), S(-6,22), S(13,18), S(66,-5), S(69,-14), S(79,-13), S(80,-12), S(82,-27),
        S(-28,19), S(1,7), S(-7,12), S(30,-4), S(15,-6), S(48,-14), S(41,-20), S(25,-21),
        S(-51,28), S(-19,24), S(-11,21), S(-7,16), S(-0,6), S(9,11), S(6,7), S(13,-16),
        S(-59,14), S(-53,23), S(-28,16), S(-6,-0), S(-2,-2), S(-6,4), S(20,-19), S(-14,-13),
        S(-50,2), S(-29,11), S(-28,7), S(-34,10), S(-6,-1), S(-24,-4), S(6,-20), S(-21,-13),
        S(-52,-9), S(-35,-14), S(-27,-1), S(-23,-1), S(-11,-18), S(-10,-9), S(18,-52), S(-29,-18),
        S(-53,-7), S(-32,-10), S(-23,-7), S(-11,-12), S(2,-22), S(-29,-3), S(-13,-24), S(-41,-28),
    ],
    [
        S(17,-45), S(-23,33), S(25,-25), S(49,2), S(48,-3), S(150,-72), S(128,-65), S(146,-127),
        S(-21,-11), S(-62,38), S(-49,63), S(-32,42), S(-26,68), S(33,60), S(-36,88), S(97,-40),
        S(-16,-43), S(-46,25), S(-45,68), S(-39,71), S(-27,95), S(32,75), S(15,62), S(26,21),
        S(-11,-28), S(-28,22), S(-29,45), S(-39,108), S(-15,91), S(-29,84), S(-3,37), S(3,-7),
        S(-5,-52), S(-22,20), S(-25,64), S(-22,77), S(-19,73), S(-24,76), S(13,2), S(15,-17),
        S(-28,-29), S(-19,-8), S(-26,52), S(-31,44), S(-21,50), S(-16,50), S(-1,-11), S(19,-61),
        S(2,-64), S(-4,-41), S(-13,-37), S(-9,-21), S(-12,-7), S(1,-29), S(23,-73), S(8,-68),
        S(-14,-52), S(-4,-87), S(-10,-91), S(-10,-38), S(3,-69), S(-8,-91), S(4,-156), S(61,-138),
    ],
    [
        S(-330,60), S(-293,163), S(1148,-396), S(-903,266), S(-956,295), S(856,-343), S(-525,180), S(-582,65),
        S(85,-13), S(243,-30), S(334,-69), S(100,-4), S(-17,15), S(229,-60), S(305,-35), S(63,2),
        S(57,-28), S(224,-10), S(181,24), S(84,22), S(161,10), S(157,31), S(293,-20), S(147,-44),
        S(30,-15), S(151,-4), S(82,8), S(110,-17), S(93,-5), S(38,17), S(81,6), S(-96,-14),
        S(-89,1), S(69,-5), S(62,-3), S(20,-12), S(21,-5), S(-22,18), S(12,4), S(-116,-0),
        S(-101,-4), S(-5,-12), S(-20,3), S(7,-24), S(22,-20), S(-45,7), S(-30,6), S(-120,1),
        S(-63,3), S(-25,1), S(-69,14), S(-78,-7), S(-80,3), S(-84,18), S(-25,1), S(-54,-15),
        S(-124,33), S(-64,10), S(-71,-4), S(-139,-11), S(-66,-46), S(-142,8), S(-51,-5), S(-81,-15),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-95,-166), S(-56,-31), S(-21,-13), S(-5,25), S(14,29), S(21,49), S(34,39), S(46,44), S(63,23), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-61,-90), S(-81,-80), S(-47,-37), S(-33,5), S(-23,19), S(-13,32), S(-7,43), S(4,29), S(4,46), S(10,33), S(10,49), S(49,4), S(45,16), S(141,-69), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(211,0), S(-45,-162), S(-60,-54), S(-50,-27), S(-43,-12), S(-36,3), S(-29,16), S(-27,24), S(-17,26), S(-10,29), S(-3,34), S(4,41), S(15,44), S(20,39), S(72,-2), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-245,-408), S(-245,-408), S(-431,-298), S(-7,79), S(-45,-215), S(-57,44), S(-63,72), S(-47,103), S(-40,81), S(-38,122), S(-31,130), S(-28,138), S(-20,129), S(-15,136), S(-12,140), S(-9,135), S(-8,137), S(-0,131), S(0,119), S(29,89), S(30,73), S(75,28), S(70,32), S(82,8), S(67,-7), S(277,-144), S(173,-154), S(536,-295)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-33,-49), S(-27,-32), S(-13,-6), S(15,21), S(15,76), S(120,172), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(8,52), S(-17,39), S(-7,9), S(-2,-4), S(6,-14), S(25,-30), S(35,-52)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-68,-14), S(1,8), S(0,24), S(9,36), S(13,44), S(9,54), S(-17,67)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-8,-22), S(9,-61), S(5,-98), S(-75,-212)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-16,23), S(-22,55), S(6,81), S(1,121)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(9,12), S(17,31), S(24,35), S(38,90), S(81,190), S(335,437), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(30,24), S(20,21), S(24,31), S(38,67), S(134,75), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(31,-11);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(44,9);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(111,-13);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(58,8);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(14,21), S(2,37), S(-9,31), S(-3,60)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-62,50), S(-50,-11), S(-45,-13), S(-39,-12), S(-17,-19), S(19,-37), S(72,-60), S(123,-84), S(176,-105), S(232,-132), S(363,-177), S(323,-144), S(272,3), S(367,2)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(73,-17), S(-20,48), S(-27,31), S(3,6), S(25,-14), S(-13,8), S(-20,-10), S(0,0)],
    [S(37,7), S(-44,22), S(-29,15), S(-4,4), S(5,-4), S(-30,-4), S(34,-69), S(0,0)],
    [S(20,11), S(-22,-0), S(8,-5), S(9,-4), S(0,-11), S(-16,-16), S(-20,-69), S(0,0)],
    [S(7,14), S(7,-28), S(-8,8), S(-2,1), S(7,-14), S(28,-18), S(76,-57), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(15,16), S(-96,-25), S(28,-44), S(19,-15), S(7,5), S(-7,17), S(4,5), S(0,0)],
    [S(-7,15), S(42,-81), S(66,-60), S(6,-3), S(-9,10), S(-35,29), S(-28,25), S(0,0)],
    [S(-12,11), S(72,-83), S(112,-62), S(23,-12), S(1,5), S(-10,13), S(-14,15), S(0,0)],
    [S(-3,2), S(104,-103), S(70,-67), S(5,-1), S(-3,5), S(-14,9), S(-9,-1), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-18,-117), S(67,40), S(55,51), S(71,45), S(76,25), S(0,0)],
    [S(-9,-93), S(186,264), S(173,362), S(166,676), S(21,2043), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(7,44), S(-52,-11), S(40,23), S(53,43), S(80,32), S(0,0)],
        [S(-4,14), S(-46,-16), S(30,42), S(18,64), S(41,82), S(0,0)],
    ],
    [
        [S(29,60), S(47,56), S(130,223), S(60,643), S(115,2065), S(0,0)],
        [S(-8,11), S(-40,-17), S(46,63), S(51,325), S(123,1462), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(-3,47), S(71,40), S(-0,-9), S(79,52), S(122,-15), S(0,0)],
        [S(-6,15), S(27,40), S(-26,-35), S(9,126), S(86,113), S(0,0)],
    ],
    [
        [S(17,80), S(129,182), S(100,70), S(100,720), S(139,2098), S(0,0)],
        [S(-5,10), S(27,60), S(-22,-17), S(72,321), S(208,1382), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(2,46), S(64,49), S(82,25), S(-55,-253), S(144,-58), S(0,0)],
        [S(-12,20), S(19,4), S(30,8), S(-73,-258), S(101,-4), S(0,0)],
    ],
    [
        [S(13,84), S(95,241), S(172,285), S(52,421), S(383,1980), S(0,0)],
        [S(-13,6), S(26,-8), S(22,8), S(-63,-262), S(303,779), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(16,-15), S(36,17), S(42,76), S(60,17), S(-63,-227), S(0,0)],
        [S(-6,-12), S(6,-21), S(-2,31), S(-8,1), S(-114,-249), S(0,0)],
    ],
    [
        [S(27,60), S(138,211), S(213,197), S(132,774), S(239,1234), S(0,0)],
        [S(-3,-4), S(1,-17), S(-4,37), S(-5,-22), S(-89,-281), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(17,11), S(24,10)];
#[rustfmt::skip]
const TEMPO: i32 = 20;

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
