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
    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn push_threat(stm: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(77,120), S(324,222), S(413,275), S(504,474), S(986,964), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(90,62), S(-0,70), S(69,60), S(118,25), S(92,44), S(3,53), S(62,18), S(20,64),
        S(2,32), S(-11,34), S(14,14), S(47,-1), S(39,-3), S(7,8), S(4,22), S(-14,31),
        S(-27,9), S(-17,-0), S(-9,-15), S(4,-36), S(9,-29), S(-25,-12), S(-5,-13), S(-12,-21),
        S(-39,3), S(-35,9), S(-7,-28), S(-4,-30), S(-4,-43), S(-0,-34), S(-20,-22), S(-30,-14),
        S(-40,-6), S(-18,-18), S(-9,-31), S(-22,-19), S(-4,-31), S(-25,-31), S(5,-38), S(-31,-23),
        S(-41,14), S(-10,-3), S(-13,-16), S(-40,8), S(-35,-9), S(-23,-7), S(10,-29), S(-24,-20),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-190,-17), S(20,3), S(-27,-42), S(-4,-38), S(-46,-41), S(-173,9), S(135,-75), S(-59,-109),
        S(-43,16), S(2,2), S(-20,-6), S(42,-16), S(7,15), S(-13,-3), S(-22,-6), S(-52,9),
        S(34,-11), S(-35,6), S(1,21), S(41,29), S(34,15), S(-6,57), S(15,13), S(5,-0),
        S(42,-5), S(18,8), S(26,13), S(50,32), S(24,33), S(40,19), S(20,20), S(36,21),
        S(8,-19), S(28,-20), S(26,37), S(12,47), S(26,53), S(31,30), S(23,12), S(26,25),
        S(-17,12), S(6,-18), S(-12,48), S(10,31), S(2,55), S(-3,38), S(26,-4), S(-8,20),
        S(-69,-64), S(6,-29), S(-5,6), S(-5,24), S(5,-3), S(8,-4), S(-3,26), S(37,-33),
        S(-80,-84), S(-19,29), S(-25,2), S(-3,-20), S(24,-30), S(17,19), S(-12,-12), S(42,-144),
    ],
    [
        S(-14,-36), S(-51,34), S(-42,-7), S(-53,-30), S(-92,-1), S(-138,21), S(-165,50), S(-27,53),
        S(13,-45), S(3,-0), S(-72,27), S(-6,-16), S(-43,-4), S(18,-3), S(-76,22), S(-58,21),
        S(39,-54), S(43,-22), S(21,6), S(24,-29), S(48,1), S(6,35), S(25,-6), S(13,11),
        S(44,-27), S(16,-1), S(37,4), S(46,11), S(18,33), S(6,20), S(10,12), S(44,-13),
        S(31,-14), S(13,-15), S(30,-7), S(21,39), S(18,37), S(9,22), S(35,18), S(12,14),
        S(15,7), S(58,-17), S(4,1), S(11,9), S(3,16), S(31,5), S(30,-40), S(16,3),
        S(44,-14), S(5,0), S(27,-18), S(-4,-9), S(10,-14), S(20,-31), S(31,-31), S(22,-34),
        S(-30,33), S(22,-22), S(3,1), S(-63,8), S(2,-41), S(-8,-5), S(-25,-6), S(-30,36),
    ],
    [
        S(10,38), S(146,-33), S(164,-63), S(87,-52), S(104,-46), S(66,-25), S(-43,49), S(-1,42),
        S(8,25), S(38,-9), S(96,-27), S(82,-19), S(131,-38), S(74,-15), S(28,2), S(-36,43),
        S(-46,35), S(5,3), S(2,3), S(45,-4), S(20,2), S(7,1), S(38,-6), S(-36,46),
        S(-4,14), S(-42,21), S(-14,28), S(42,-6), S(-5,6), S(-4,19), S(-16,40), S(5,1),
        S(-36,12), S(-37,8), S(6,14), S(-48,28), S(7,-13), S(-26,3), S(-7,-10), S(-52,22),
        S(-57,6), S(-8,-15), S(-26,-4), S(-53,16), S(-12,-7), S(-21,-26), S(-34,18), S(-59,-1),
        S(-68,-6), S(-43,-1), S(-24,-17), S(-13,-30), S(-12,-36), S(-27,1), S(-18,-21), S(-78,-21),
        S(-51,12), S(-36,1), S(-21,5), S(-17,1), S(11,-20), S(-10,-4), S(-46,19), S(-36,-7),
    ],
    [
        S(5,30), S(36,-2), S(-71,123), S(-7,36), S(30,23), S(59,-8), S(-49,88), S(35,-11),
        S(4,-27), S(-41,45), S(-47,51), S(-22,31), S(14,29), S(-17,102), S(15,-35), S(38,-16),
        S(-19,34), S(-20,19), S(-21,36), S(-56,124), S(-22,71), S(51,46), S(3,67), S(35,-20),
        S(16,-8), S(17,23), S(-33,99), S(-17,99), S(2,74), S(-17,54), S(15,2), S(39,-57),
        S(11,-65), S(2,-51), S(15,26), S(-7,33), S(-3,49), S(-4,14), S(27,-3), S(28,-32),
        S(14,-101), S(3,-22), S(15,-31), S(0,-11), S(1,-35), S(-2,14), S(11,-15), S(13,-102),
        S(-38,63), S(-3,-53), S(18,-84), S(11,-63), S(1,-22), S(30,-109), S(17,-51), S(-10,-77),
        S(-24,35), S(15,-108), S(23,-88), S(11,-48), S(28,-101), S(7,-122), S(-74,-21), S(-95,62),
    ],
    [
        S(24,-148), S(-285,-11), S(346,-94), S(170,-14), S(-231,155), S(-407,183), S(-13,-18), S(1,-22),
        S(-390,59), S(-8,32), S(10,29), S(-354,147), S(-269,174), S(-377,129), S(6,24), S(79,-38),
        S(-20,27), S(-65,51), S(-138,66), S(-197,62), S(-39,23), S(-309,105), S(-322,139), S(-373,157),
        S(-52,18), S(115,4), S(102,6), S(168,-53), S(376,-76), S(138,-5), S(-2,29), S(4,13),
        S(42,-58), S(192,-40), S(98,-23), S(136,-23), S(203,-44), S(133,-16), S(75,-22), S(-42,-8),
        S(96,-43), S(134,-55), S(42,-17), S(84,-24), S(11,-5), S(88,-25), S(96,-48), S(8,-23),
        S(103,-64), S(67,-42), S(90,-37), S(46,-21), S(38,-17), S(36,-17), S(82,-31), S(62,-42),
        S(22,-63), S(65,-34), S(68,-40), S(1,-27), S(79,-75), S(33,-53), S(72,-45), S(54,-71),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-160,-349), S(-21,20), S(-7,37), S(7,43), S(19,57), S(25,54), S(31,62), S(44,64), S(62,12), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-64,-97), S(-30,-95), S(-47,-32), S(-37,18), S(-24,27), S(-21,50), S(-18,54), S(-12,48), S(-7,52), S(9,33), S(-0,47), S(46,-21), S(26,17), S(180,-101), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-398,-401), S(-29,-29), S(-25,-47), S(-21,-4), S(-10,19), S(-1,40), S(7,43), S(11,52), S(17,59), S(26,64), S(30,64), S(26,72), S(62,59), S(58,65), S(247,-56), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-308,-270), S(-308,-270), S(-683,-374), S(-136,451), S(-45,-151), S(-47,39), S(-43,75), S(-35,84), S(-33,106), S(-28,107), S(-30,128), S(-24,143), S(-21,137), S(-23,153), S(-23,153), S(-27,148), S(-22,135), S(-10,114), S(-23,105), S(-2,100), S(-7,92), S(39,16), S(85,-25), S(193,-122), S(266,-175), S(354,-291), S(537,-299), S(402,-308)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-16,-46), S(-23,-16), S(-24,15), S(5,30), S(16,49), S(113,123), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-31,54), S(-11,35), S(15,2), S(8,3), S(5,17), S(26,12), S(15,13)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-13,-41), S(7,-8), S(1,18), S(2,39), S(-4,55), S(19,36), S(-24,60)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-24,-9), S(7,-48), S(15,-99), S(-143,-168)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-13,25), S(-15,62), S(-6,86), S(67,95)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,4), S(5,17), S(14,23), S(41,58), S(57,232), S(508,441), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(11,17), S(10,13), S(13,15), S(46,37), S(489,-119), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(29,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(18,11);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(87,-19);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(27,35);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(13,27), S(2,26), S(17,24), S(-4,71)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-47,54), S(-44,-1), S(-43,7), S(-36,4), S(-16,-14), S(9,-18), S(54,-50), S(88,-79), S(173,-107), S(160,-76), S(278,-242), S(285,-186), S(217,-200), S(298,-390)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(38,0), S(-22,16), S(-38,17), S(-18,13), S(4,-13), S(-55,2), S(-66,26), S(0,0)],
    [S(37,-6), S(-33,6), S(-23,8), S(5,-2), S(20,-15), S(19,-18), S(-85,14), S(0,0)],
    [S(31,2), S(-24,-1), S(6,-16), S(6,0), S(-20,17), S(-7,8), S(-66,5), S(0,0)],
    [S(23,-1), S(-13,-2), S(1,-8), S(-0,-3), S(4,-5), S(34,-18), S(83,-17), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-15,-57), S(77,39), S(62,61), S(44,54), S(15,177), S(0,0)],
    [S(-11,-43), S(207,172), S(236,210), S(249,449), S(462,1426), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,62), S(18,-129), S(46,31), S(65,35), S(38,160), S(0,0)],
        [S(-2,16), S(-2,-160), S(36,49), S(61,25), S(19,249), S(0,0)],
    ],
    [
        [S(26,64), S(71,-101), S(153,162), S(170,398), S(202,1291), S(0,0)],
        [S(-4,15), S(7,-156), S(46,50), S(78,215), S(192,1019), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,47), S(59,20), S(-20,5), S(41,62), S(99,45), S(0,0)],
        [S(5,9), S(26,36), S(-26,-3), S(43,95), S(58,213), S(0,0)],
    ],
    [
        [S(29,64), S(157,138), S(98,24), S(210,442), S(404,1283), S(0,0)],
        [S(-1,8), S(23,49), S(-28,-25), S(92,215), S(388,415), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,64), S(63,51), S(82,52), S(38,-228), S(92,74), S(0,0)],
        [S(-5,29), S(12,28), S(33,25), S(5,-215), S(95,131), S(0,0)],
    ],
    [
        [S(2,85), S(103,213), S(157,199), S(184,181), S(488,1359), S(0,0)],
        [S(-6,15), S(24,4), S(26,10), S(11,-242), S(278,730), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(17,-14), S(36,9), S(48,101), S(88,-12), S(-50,-199), S(0,0)],
        [S(-3,10), S(-2,2), S(10,16), S(10,-23), S(-112,-163), S(0,0)],
    ],
    [
        [S(25,68), S(122,55), S(187,119), S(302,212), S(427,939), S(0,0)],
        [S(-2,0), S(-6,-0), S(-15,50), S(-2,7), S(-88,-208), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(12,3), S(24,-2)];
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
    let rank = if file_pawns.any() {
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
    return Params::pawn_shield(edge_dist, rank);
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
