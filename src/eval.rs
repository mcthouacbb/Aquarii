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
const MATERIAL: [ScorePair; 6] = [S(97,134), S(453,250), S(538,254), S(706,548), S(1165,1114), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(101,10), S(51,51), S(73,30), S(69,38), S(62,45), S(23,42), S(-22,59), S(69,23),
        S(6,23), S(1,23), S(-19,9), S(17,-3), S(47,-12), S(2,6), S(-23,12), S(-14,27),
        S(-9,13), S(-6,-2), S(-8,-23), S(1,-28), S(18,-30), S(-10,-34), S(-17,-6), S(-10,3),
        S(-24,2), S(-20,-3), S(-5,-30), S(-6,-31), S(1,-33), S(-11,-25), S(-12,-19), S(-25,-4),
        S(-43,3), S(-25,-7), S(-21,-22), S(-29,-24), S(-11,-15), S(-22,-26), S(-13,-18), S(-50,2),
        S(-28,11), S(-12,7), S(-8,-9), S(-13,-18), S(-0,-16), S(-3,-10), S(7,-2), S(-30,10),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-94,-70), S(39,-80), S(21,-113), S(-56,14), S(6,3), S(77,-172), S(104,-110), S(4,-77),
        S(42,-90), S(3,-7), S(-18,7), S(15,-27), S(72,-50), S(12,-4), S(-7,-28), S(106,-104),
        S(36,-26), S(21,-45), S(-15,52), S(3,54), S(22,51), S(66,33), S(80,-70), S(-2,1),
        S(12,21), S(0,39), S(9,56), S(30,49), S(13,69), S(6,78), S(21,51), S(46,21),
        S(-2,-12), S(3,18), S(-2,62), S(2,56), S(-9,61), S(8,60), S(5,33), S(25,6),
        S(-33,-9), S(-28,17), S(-31,35), S(-16,52), S(-9,57), S(-27,46), S(-14,21), S(-22,-3),
        S(-33,-42), S(-46,71), S(-41,7), S(-31,28), S(-29,23), S(-23,12), S(-9,66), S(-1,-1),
        S(-87,-98), S(-46,-19), S(-38,-7), S(-19,19), S(-32,39), S(-31,22), S(-30,-14), S(-28,-133),
    ],
    [
        S(-26,-38), S(58,-32), S(8,-57), S(-84,-7), S(12,-40), S(-67,-47), S(112,-70), S(-34,-58),
        S(26,-58), S(-21,-1), S(-17,9), S(-7,-36), S(-27,-30), S(-4,14), S(-110,7), S(19,-79),
        S(11,-1), S(3,21), S(-11,12), S(15,21), S(35,8), S(35,10), S(47,-5), S(34,-7),
        S(-13,8), S(-4,34), S(-1,31), S(4,47), S(-9,59), S(13,20), S(-8,29), S(0,-19),
        S(-5,-19), S(-1,10), S(2,30), S(1,46), S(3,29), S(-10,35), S(7,2), S(23,-31),
        S(-11,-14), S(10,8), S(-10,26), S(-1,28), S(-11,37), S(-4,19), S(10,18), S(19,-43),
        S(4,-11), S(-3,-3), S(3,-1), S(-24,23), S(-15,31), S(9,3), S(14,2), S(11,-28),
        S(21,7), S(11,-4), S(-22,6), S(-19,8), S(-14,16), S(-16,19), S(34,11), S(-3,-3),
    ],
    [
        S(7,5), S(-17,48), S(14,34), S(35,9), S(21,5), S(18,29), S(81,9), S(72,-24),
        S(-1,17), S(4,12), S(19,14), S(58,-5), S(73,-16), S(52,-17), S(75,-21), S(87,-38),
        S(-7,9), S(25,-2), S(9,6), S(32,-6), S(20,-5), S(40,-9), S(26,-18), S(56,-43),
        S(-39,24), S(-13,21), S(-22,26), S(2,13), S(14,9), S(22,1), S(46,-8), S(-13,-7),
        S(-60,6), S(-30,14), S(-22,14), S(-8,7), S(7,-5), S(-0,-4), S(-3,-3), S(-33,-8),
        S(-61,15), S(-31,19), S(-39,20), S(-21,5), S(-7,-1), S(-24,-0), S(-6,-19), S(-28,-0),
        S(-50,3), S(-40,-10), S(-29,0), S(-13,-7), S(-11,-13), S(-22,1), S(-19,-28), S(-35,-25),
        S(-51,-6), S(-36,3), S(-18,-1), S(-10,-15), S(-7,-10), S(-28,-2), S(-17,-7), S(-41,-15),
    ],
    [
        S(-11,-37), S(-1,-3), S(94,-74), S(52,-36), S(110,-61), S(79,-19), S(91,-64), S(142,-91),
        S(-7,-21), S(-53,45), S(-49,69), S(-16,35), S(-39,81), S(32,61), S(-21,47), S(59,8),
        S(-27,-20), S(-31,-4), S(-27,49), S(-22,63), S(-13,104), S(-3,75), S(36,43), S(17,-5),
        S(-12,-27), S(-30,48), S(-36,56), S(-24,83), S(-6,86), S(-14,75), S(-15,60), S(-4,16),
        S(-10,-47), S(-22,6), S(-27,70), S(-33,88), S(-24,70), S(-7,48), S(14,4), S(10,-34),
        S(-25,-8), S(-16,7), S(-25,62), S(-30,48), S(-20,46), S(-12,34), S(-4,-16), S(5,-23),
        S(-7,-46), S(-4,-29), S(-7,-27), S(-6,-19), S(-18,-17), S(-10,-34), S(1,-65), S(25,-113),
        S(-17,-47), S(-0,-101), S(-13,-66), S(-9,-36), S(4,-73), S(4,-85), S(-26,-112), S(55,-125),
    ],
    [
        S(-182,44), S(-175,108), S(492,-195), S(-263,105), S(-249,105), S(502,-164), S(-339,119), S(-378,54),
        S(-5,-10), S(229,-29), S(228,-51), S(19,-5), S(-18,8), S(164,-47), S(227,-40), S(-45,2),
        S(131,-74), S(259,-34), S(27,12), S(133,4), S(45,23), S(59,30), S(184,-40), S(304,-98),
        S(61,-25), S(136,-4), S(73,3), S(29,0), S(30,12), S(86,8), S(70,3), S(8,-23),
        S(-12,-12), S(71,-4), S(61,-4), S(13,-5), S(65,-14), S(30,-2), S(36,-1), S(-111,2),
        S(-114,7), S(-4,-11), S(-23,-2), S(36,-31), S(24,-21), S(-24,-3), S(-44,11), S(-121,3),
        S(-88,19), S(-63,13), S(-87,12), S(-123,17), S(-108,19), S(-115,21), S(-50,12), S(-83,5),
        S(-163,57), S(-97,34), S(-106,14), S(-184,16), S(-110,-16), S(-167,22), S(-80,21), S(-100,20),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-142,-197), S(-60,24), S(-21,16), S(-2,36), S(24,22), S(30,36), S(44,26), S(56,33), S(72,2), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-43,-199), S(-58,-63), S(-57,5), S(-41,33), S(-29,41), S(-19,51), S(-13,60), S(2,36), S(-2,53), S(23,29), S(10,41), S(75,-20), S(46,-3), S(106,-64), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(441,-401), S(11,-159), S(-74,-24), S(-68,18), S(-61,25), S(-57,39), S(-53,49), S(-47,53), S(-42,56), S(-34,58), S(-31,66), S(-24,68), S(-20,71), S(-5,58), S(63,22), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-278,-329), S(-278,-329), S(-217,-219), S(-73,-47), S(-117,319), S(-81,168), S(-52,19), S(-53,91), S(-53,97), S(-48,90), S(-41,105), S(-38,114), S(-32,105), S(-30,112), S(-26,105), S(-19,108), S(-19,97), S(-11,96), S(-12,86), S(10,67), S(46,17), S(51,8), S(78,-24), S(73,-29), S(153,-92), S(296,-190), S(163,-167), S(606,-376)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-47,-32), S(-34,-20), S(-17,2), S(6,23), S(37,46), S(182,117), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(5,46), S(-13,30), S(0,3), S(8,-9), S(15,-21), S(26,-38), S(45,-58)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-25,-18), S(15,1), S(-2,18), S(3,24), S(9,39), S(9,46), S(-28,66)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-9,-17), S(6,-54), S(-16,-90), S(-83,-192)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-13,21), S(-21,50), S(4,90), S(-8,136)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(12,9), S(17,35), S(28,26), S(45,67), S(114,160), S(217,302), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(35,16), S(25,15), S(29,27), S(46,52), S(201,47), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(34,-14);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(58,-11);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(95,-12);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(47,7);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(17,19), S(1,42), S(-10,38), S(-3,61)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-43,45), S(-36,-16), S(-38,-16), S(-35,-9), S(-27,-12), S(1,-24), S(43,-47), S(91,-69), S(120,-69), S(210,-132), S(200,-79), S(260,-135), S(328,-130), S(591,-628)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(59,-19), S(-19,41), S(-22,36), S(4,15), S(16,-1), S(-21,30), S(-42,-23), S(0,0)],
    [S(37,1), S(-42,20), S(-28,6), S(-1,-7), S(10,-4), S(-44,-6), S(-79,-26), S(0,0)],
    [S(18,11), S(-26,0), S(5,-10), S(7,-5), S(7,-16), S(16,-25), S(33,-72), S(0,0)],
    [S(12,17), S(1,-4), S(-10,8), S(-9,-5), S(8,-14), S(-6,-14), S(59,-63), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(16,16), S(-147,9), S(19,-45), S(14,-14), S(2,2), S(3,15), S(7,9), S(0,0)],
    [S(-2,6), S(-160,-0), S(37,-47), S(-0,-6), S(-13,15), S(-21,12), S(-18,14), S(0,0)],
    [S(-5,10), S(-9,-50), S(45,-35), S(17,-0), S(4,3), S(4,14), S(-6,18), S(0,0)],
    [S(-7,3), S(14,-70), S(28,-47), S(0,-4), S(-2,-3), S(-2,-4), S(-7,-1), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-27,-101), S(81,20), S(66,41), S(57,47), S(81,-3), S(0,0)],
    [S(-15,-80), S(237,210), S(258,252), S(248,485), S(383,1643), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,40), S(-70,24), S(53,7), S(75,38), S(79,7), S(0,0)],
        [S(-4,12), S(-48,-9), S(39,21), S(47,34), S(44,10), S(0,0)],
    ],
    [
        [S(37,63), S(28,93), S(171,162), S(138,523), S(370,1620), S(0,0)],
        [S(-8,17), S(-42,-0), S(52,48), S(106,230), S(207,1114), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(4,54), S(69,58), S(-19,-13), S(102,33), S(126,43), S(0,0)],
        [S(-2,13), S(31,39), S(-28,-41), S(26,98), S(88,137), S(0,0)],
    ],
    [
        [S(20,93), S(173,130), S(110,80), S(170,562), S(236,1738), S(0,0)],
        [S(0,10), S(37,38), S(-28,-31), S(124,282), S(207,1136), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(4,41), S(63,26), S(104,-14), S(-48,-161), S(138,-30), S(0,0)],
        [S(-12,14), S(22,-7), S(27,3), S(-78,-161), S(81,27), S(0,0)],
    ],
    [
        [S(15,85), S(140,187), S(197,216), S(169,340), S(598,1563), S(0,0)],
        [S(-12,8), S(28,-19), S(29,-5), S(-64,-161), S(337,652), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(16,-7), S(45,20), S(68,27), S(74,-6), S(-92,-185), S(0,0)],
        [S(-5,-13), S(7,-20), S(-4,15), S(-8,-1), S(-118,-235), S(0,0)],
    ],
    [
        [S(31,54), S(188,119), S(266,137), S(194,726), S(437,803), S(0,0)],
        [S(-1,-7), S(3,-15), S(-3,25), S(-2,-16), S(-107,-234), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(17,2), S(26,6)];
#[rustfmt::skip]
const TEMPO: i32 = 14;

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
