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
const MATERIAL: [ScorePair; 6] = [S(73,129), S(320,215), S(411,272), S(502,476), S(990,957), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(98,56), S(-1,71), S(86,49), S(95,38), S(63,59), S(7,60), S(38,35), S(52,66),
        S(8,36), S(-13,38), S(2,15), S(43,2), S(38,-5), S(-89,36), S(-8,37), S(-2,49),
        S(-26,12), S(-14,-2), S(-10,-14), S(5,-39), S(12,-37), S(-27,-8), S(-2,-14), S(-4,-13),
        S(-36,2), S(-32,5), S(-7,-28), S(-1,-43), S(-2,-56), S(-5,-27), S(-15,-25), S(-21,-14),
        S(-37,-8), S(-15,-21), S(-9,-31), S(-17,-26), S(-1,-40), S(-20,-30), S(18,-43), S(-22,-25),
        S(-36,12), S(-7,-7), S(-14,-20), S(-31,-8), S(-25,-26), S(-20,-15), S(18,-36), S(-17,-17),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-182,-8), S(28,-17), S(-58,-33), S(3,-46), S(-58,-46), S(-172,12), S(214,-120), S(-56,-98),
        S(-48,20), S(31,-4), S(-20,-8), S(34,-7), S(25,8), S(-11,-7), S(-6,-11), S(-66,32),
        S(35,-20), S(-33,5), S(-3,18), S(43,21), S(29,22), S(-1,52), S(14,18), S(21,-8),
        S(40,-7), S(16,5), S(22,18), S(48,32), S(23,27), S(41,19), S(22,19), S(34,21),
        S(6,-21), S(31,-29), S(24,36), S(11,46), S(24,50), S(29,33), S(23,14), S(24,28),
        S(-19,7), S(2,-16), S(-14,44), S(5,32), S(-1,48), S(-6,32), S(22,-6), S(-9,13),
        S(-75,-61), S(-1,-15), S(-13,15), S(-9,22), S(2,-6), S(3,-1), S(-7,29), S(35,-29),
        S(-89,-50), S(-20,34), S(-33,-1), S(-11,-9), S(12,-14), S(7,21), S(-15,-10), S(56,-144),
    ],
    [
        S(-23,-37), S(-86,36), S(-35,-9), S(-47,-23), S(-122,4), S(-157,27), S(-177,67), S(-27,64),
        S(22,-51), S(-1,6), S(-69,19), S(2,-19), S(-46,-6), S(5,4), S(-69,18), S(-58,15),
        S(35,-53), S(52,-25), S(26,7), S(26,-30), S(45,1), S(9,35), S(28,-7), S(19,8),
        S(52,-29), S(16,-1), S(38,4), S(47,11), S(21,29), S(5,20), S(12,12), S(45,-13),
        S(36,-17), S(13,-17), S(30,-6), S(24,36), S(17,34), S(12,17), S(33,23), S(8,13),
        S(14,4), S(61,-22), S(6,0), S(12,8), S(4,14), S(33,6), S(31,-34), S(19,4),
        S(43,-11), S(7,-2), S(24,-21), S(-2,-13), S(13,-19), S(20,-30), S(33,-31), S(32,-36),
        S(-23,22), S(18,-19), S(4,-2), S(-60,16), S(1,-34), S(-8,-6), S(-22,-6), S(-25,50),
    ],
    [
        S(1,41), S(143,-29), S(162,-61), S(106,-55), S(95,-39), S(51,-22), S(-31,43), S(3,37),
        S(7,25), S(40,-15), S(95,-24), S(70,-13), S(124,-29), S(88,-19), S(31,-1), S(-39,47),
        S(-41,31), S(1,3), S(1,5), S(44,-6), S(19,-0), S(19,-5), S(39,-4), S(-32,47),
        S(1,10), S(-39,16), S(-6,22), S(39,-11), S(1,1), S(-7,25), S(-12,39), S(0,1),
        S(-37,9), S(-36,7), S(12,8), S(-45,28), S(8,-10), S(-16,-7), S(-2,-5), S(-51,18),
        S(-61,4), S(-14,-16), S(-26,-9), S(-51,13), S(-7,-21), S(-25,-19), S(-36,13), S(-59,4),
        S(-70,-2), S(-44,2), S(-25,-8), S(-13,-31), S(-16,-34), S(-30,10), S(-21,-18), S(-78,-22),
        S(-54,15), S(-42,7), S(-22,4), S(-20,4), S(4,-15), S(-14,-2), S(-48,23), S(-38,-4),
    ],
    [
        S(14,37), S(42,-9), S(-61,101), S(-10,32), S(54,-12), S(105,-66), S(-60,115), S(45,-25),
        S(-10,-20), S(-35,41), S(-54,63), S(5,16), S(18,32), S(-26,117), S(15,-25), S(31,-14),
        S(-22,37), S(-24,13), S(-23,41), S(-54,115), S(-18,70), S(46,50), S(7,55), S(28,-3),
        S(16,-6), S(17,16), S(-25,87), S(-23,109), S(-8,93), S(-16,59), S(17,2), S(37,-44),
        S(6,-53), S(-3,-37), S(16,13), S(-5,30), S(-8,48), S(-3,19), S(24,10), S(27,-25),
        S(1,-78), S(-0,-18), S(9,-14), S(-1,-4), S(1,-25), S(-5,18), S(8,-14), S(19,-126),
        S(-42,76), S(-8,-44), S(14,-85), S(6,-49), S(-6,-3), S(20,-80), S(7,-45), S(-10,-103),
        S(-21,16), S(15,-126), S(21,-97), S(7,-43), S(20,-101), S(11,-132), S(-69,-26), S(-75,24),
    ],
    [
        S(20,-124), S(-229,-0), S(275,-76), S(133,-20), S(-296,159), S(-329,162), S(-23,-4), S(19,-23),
        S(-395,66), S(-40,42), S(19,20), S(-365,152), S(-332,171), S(-385,112), S(27,26), S(15,-26),
        S(-66,48), S(-66,63), S(-151,65), S(-132,48), S(-46,25), S(-321,98), S(-337,148), S(-376,166),
        S(-62,26), S(127,20), S(149,-2), S(174,-55), S(377,-79), S(176,-19), S(-7,43), S(15,10),
        S(69,-53), S(198,-33), S(101,-30), S(149,-25), S(209,-47), S(120,-19), S(57,-12), S(-45,-0),
        S(106,-43), S(113,-46), S(39,-26), S(93,-25), S(24,-7), S(84,-29), S(93,-43), S(8,-24),
        S(114,-73), S(64,-43), S(91,-45), S(51,-27), S(44,-20), S(44,-27), S(85,-29), S(67,-46),
        S(28,-69), S(68,-38), S(76,-51), S(12,-35), S(91,-86), S(41,-68), S(76,-46), S(60,-76),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-188,-408), S(-21,22), S(-3,41), S(10,50), S(23,67), S(29,65), S(36,71), S(48,73), S(66,20), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-69,-122), S(-35,-90), S(-50,-27), S(-38,17), S(-26,30), S(-24,53), S(-22,57), S(-13,50), S(-9,52), S(5,38), S(-3,48), S(45,-16), S(32,11), S(208,-100), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-389,-403), S(-68,18), S(-22,-52), S(-19,-7), S(-9,21), S(2,37), S(11,41), S(14,49), S(20,55), S(28,60), S(33,61), S(30,68), S(61,55), S(59,62), S(250,-62), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-308,-267), S(-308,-267), S(-678,-356), S(-125,450), S(-42,-174), S(-52,47), S(-46,82), S(-39,82), S(-38,107), S(-32,111), S(-34,126), S(-27,135), S(-24,136), S(-26,153), S(-26,148), S(-28,144), S(-26,141), S(-19,120), S(-25,105), S(-2,101), S(-13,94), S(17,33), S(90,-28), S(234,-146), S(221,-146), S(356,-289), S(620,-333), S(380,-309)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-20,-42), S(-22,-17), S(-17,10), S(5,29), S(23,49), S(107,132), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-14,56), S(0,34), S(11,5), S(4,2), S(-1,16), S(19,14), S(5,12)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-17,-16), S(2,-2), S(4,17), S(4,37), S(-3,50), S(20,30), S(-10,48)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-22,-16), S(9,-54), S(13,-93), S(-139,-162)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-12,26), S(-12,58), S(-7,77), S(67,88)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(7,5), S(5,19), S(14,24), S(38,62), S(93,217), S(471,494), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(13,18), S(11,15), S(13,16), S(43,37), S(458,-94), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(27,-14);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(17,13);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(82,-17);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(26,36);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(13,26), S(3,27), S(7,29), S(-5,77)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-46,55), S(-42,-0), S(-42,7), S(-36,6), S(-15,-13), S(7,-18), S(48,-45), S(89,-79), S(179,-105), S(166,-87), S(306,-245), S(283,-164), S(168,-144), S(293,-395)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(39,-6), S(-17,18), S(-34,17), S(-13,11), S(4,-21), S(-48,-6), S(-128,19), S(0,0)],
    [S(41,-8), S(-31,10), S(-29,13), S(13,-6), S(26,-21), S(7,-22), S(-115,25), S(0,0)],
    [S(29,1), S(-24,3), S(2,-9), S(9,-1), S(-21,12), S(-5,-6), S(-30,-16), S(0,0)],
    [S(24,3), S(-16,1), S(-6,1), S(-0,-1), S(3,-1), S(34,-14), S(60,-13), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(11,35), S(-96,-97), S(-3,-42), S(-17,-8), S(-10,20), S(-15,28), S(-7,15), S(0,0)],
    [S(16,5), S(59,-106), S(28,-54), S(-3,-13), S(4,-9), S(-39,18), S(-14,18), S(0,0)],
    [S(-4,12), S(70,-76), S(126,-55), S(2,-18), S(9,-11), S(-13,2), S(-6,13), S(0,0)],
    [S(8,5), S(242,-131), S(18,-23), S(3,-3), S(1,14), S(-11,9), S(-10,13), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-17,-63), S(79,37), S(55,66), S(58,49), S(14,179), S(0,0)],
    [S(-11,-49), S(204,183), S(240,211), S(250,439), S(472,1404), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(7,63), S(9,-162), S(43,43), S(61,41), S(40,181), S(0,0)],
        [S(-2,18), S(-12,-174), S(33,56), S(64,25), S(21,246), S(0,0)],
    ],
    [
        [S(24,67), S(68,-120), S(150,161), S(164,394), S(225,1267), S(0,0)],
        [S(-5,17), S(-5,-175), S(45,47), S(83,213), S(203,830), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,46), S(58,20), S(-19,1), S(36,76), S(102,46), S(0,0)],
        [S(4,10), S(26,40), S(-26,-9), S(37,108), S(61,196), S(0,0)],
    ],
    [
        [S(27,66), S(153,142), S(100,23), S(212,436), S(441,1148), S(0,0)],
        [S(-2,9), S(22,50), S(-30,-21), S(88,212), S(405,358), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,62), S(63,51), S(76,52), S(-1,-253), S(106,40), S(0,0)],
        [S(-5,30), S(15,25), S(34,20), S(-20,-244), S(107,99), S(0,0)],
    ],
    [
        [S(3,85), S(99,201), S(152,200), S(145,156), S(500,1311), S(0,0)],
        [S(-6,16), S(19,9), S(26,11), S(-15,-275), S(270,727), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(17,-12), S(38,22), S(52,84), S(77,-5), S(-49,-244), S(0,0)],
        [S(-4,13), S(-0,0), S(11,13), S(6,-13), S(-116,-181), S(0,0)],
    ],
    [
        [S(24,66), S(125,63), S(186,115), S(310,192), S(437,920), S(0,0)],
        [S(-1,0), S(-6,-5), S(-15,54), S(4,-13), S(-94,-236), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(13,4), S(24,-1)];
#[rustfmt::skip]
const TEMPO: i32 = 25;

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
