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
const MATERIAL: [ScorePair; 6] = [S(100,116), S(413,182), S(468,181), S(635,380), S(1289,560), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(102,50), S(87,20), S(30,44), S(137,14), S(144,-5), S(-31,95), S(11,67), S(110,52),
        S(5,56), S(-9,26), S(-6,28), S(23,18), S(1,17), S(-57,56), S(-20,33), S(34,46),
        S(-30,12), S(-14,-10), S(-12,-14), S(-5,-27), S(-3,-29), S(-7,-12), S(-10,2), S(-15,-5),
        S(-27,-16), S(-38,-10), S(-6,-40), S(-6,-40), S(-2,-47), S(-4,-32), S(-28,-20), S(-27,-18),
        S(-24,-19), S(-26,-20), S(-25,-35), S(-30,-24), S(-15,-30), S(-40,-33), S(13,-37), S(-30,-26),
        S(-35,1), S(-20,-17), S(-21,-19), S(-37,-1), S(-25,-4), S(0,-13), S(12,-25), S(-23,-10),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-102,-27), S(-24,-25), S(-71,-1), S(2,-36), S(-60,54), S(-80,-92), S(61,-4), S(-142,42),
        S(-62,14), S(25,-5), S(-17,17), S(-11,-8), S(47,-40), S(17,-34), S(-52,41), S(-6,-62),
        S(-66,12), S(46,-13), S(-7,39), S(24,31), S(40,-1), S(73,-19), S(48,-66), S(-12,20),
        S(-24,13), S(21,-9), S(33,22), S(66,36), S(53,22), S(59,28), S(61,-28), S(53,-44),
        S(6,-6), S(58,9), S(23,40), S(24,36), S(27,21), S(45,20), S(53,-39), S(16,-20),
        S(-36,8), S(-9,10), S(-14,37), S(-1,34), S(10,43), S(-3,27), S(29,0), S(-19,-32),
        S(-20,-11), S(-42,12), S(-12,15), S(-6,23), S(-7,10), S(3,41), S(8,8), S(-18,-19),
        S(162,-90), S(-54,-2), S(-56,15), S(-39,24), S(-38,22), S(-43,14), S(-39,17), S(-2,-147),
    ],
    [
        S(-13,-3), S(-4,-21), S(-60,-15), S(-88,-47), S(-81,-41), S(-135,2), S(-76,-43), S(-81,-14),
        S(1,-10), S(4,1), S(-55,-4), S(-6,-30), S(-105,5), S(27,-19), S(-82,10), S(-26,-7),
        S(-4,24), S(-7,32), S(5,27), S(33,-3), S(73,-26), S(21,18), S(28,23), S(33,-14),
        S(-16,8), S(22,17), S(7,35), S(29,33), S(27,28), S(17,7), S(14,-5), S(38,-8),
        S(49,-4), S(3,21), S(29,25), S(5,47), S(15,28), S(6,19), S(36,-3), S(22,-44),
        S(11,-14), S(41,-18), S(9,18), S(18,14), S(1,29), S(29,13), S(24,4), S(13,21),
        S(51,-35), S(9,-8), S(27,-14), S(-7,-1), S(17,2), S(19,-22), S(33,8), S(16,-16),
        S(15,-16), S(-4,-32), S(-9,-7), S(-12,-3), S(-3,-15), S(-15,14), S(-6,17), S(12,8),
    ],
    [
        S(102,-70), S(2,42), S(-25,65), S(-5,54), S(-45,26), S(-25,44), S(49,-29), S(150,-116),
        S(17,-1), S(14,26), S(33,34), S(38,4), S(70,-12), S(47,-5), S(12,6), S(139,-87),
        S(-11,6), S(-1,28), S(55,-17), S(5,8), S(106,-94), S(67,-42), S(17,2), S(-9,-11),
        S(-26,16), S(-0,12), S(19,-6), S(17,6), S(58,-24), S(19,-9), S(2,-10), S(1,-13),
        S(-55,33), S(-68,30), S(-29,24), S(-28,17), S(-1,-12), S(13,-9), S(12,-29), S(-37,11),
        S(-58,19), S(-37,17), S(-64,19), S(-60,23), S(-15,5), S(-33,15), S(-3,-12), S(-51,-2),
        S(-59,6), S(-34,-12), S(-55,31), S(-12,-16), S(-13,-7), S(-13,-10), S(4,-37), S(-72,2),
        S(-44,28), S(-25,10), S(-15,19), S(-8,10), S(21,-2), S(0,3), S(-34,17), S(-20,-20),
    ],
    [
        S(-3,-81), S(-41,4), S(78,-56), S(132,-154), S(70,-124), S(76,39), S(71,-127), S(67,-69),
        S(22,-120), S(-55,82), S(-13,51), S(3,18), S(-20,96), S(36,-24), S(-8,43), S(76,-177),
        S(4,-60), S(-33,48), S(-15,51), S(-29,99), S(-8,79), S(58,100), S(14,4), S(45,-2),
        S(-25,12), S(-30,31), S(-48,100), S(-34,120), S(-20,107), S(-10,85), S(-21,83), S(28,-50),
        S(17,-113), S(-39,65), S(-17,108), S(-38,107), S(-18,93), S(-14,40), S(3,46), S(1,15),
        S(-22,-22), S(-5,-4), S(-48,105), S(-7,-14), S(-21,31), S(-10,69), S(1,-7), S(-11,-24),
        S(-46,68), S(-14,-37), S(-3,-81), S(-2,-29), S(0,8), S(-4,-21), S(-8,-15), S(-59,8),
        S(39,-179), S(-13,-58), S(8,-95), S(-2,-40), S(15,-122), S(-57,54), S(-17,-65), S(20,-97),
    ],
    [
        S(2260,110), S(2641,40), S(410,120), S(1402,55), S(-326,-13), S(-453,-73), S(2558,-10), S(2600,-30),
        S(669,29), S(-44,42), S(-56,44), S(-611,-3), S(-164,-175), S(-580,90), S(-57,15), S(1134,35),
        S(8,-14), S(-226,25), S(-81,-25), S(-183,10), S(-185,10), S(-351,-7), S(-214,-59), S(-160,-78),
        S(-245,1), S(-244,-1), S(-103,-7), S(-171,32), S(-220,39), S(-263,23), S(-238,-2), S(-357,14),
        S(-495,25), S(-214,-3), S(-250,14), S(-127,16), S(-179,35), S(-209,3), S(-347,23), S(-439,6),
        S(-320,-9), S(-296,-5), S(-297,28), S(-195,38), S(-215,23), S(-255,16), S(-318,22), S(-328,3),
        S(-249,-28), S(-280,-3), S(-185,-19), S(-214,23), S(-210,15), S(-233,-6), S(-254,-7), S(-250,-35),
        S(-294,-56), S(-248,-40), S(-158,-55), S(-217,-26), S(-143,-54), S(-231,-42), S(-248,-44), S(-255,-94),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-55,63), S(-31,-20), S(-13,-19), S(-1,-12), S(6,-6), S(9,4), S(16,2), S(32,2), S(39,-12), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(10,-306), S(-52,-48), S(-47,5), S(-28,32), S(-15,37), S(-8,47), S(-4,49), S(-3,46), S(4,53), S(17,32), S(16,50), S(35,9), S(40,4), S(34,-10), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(172,18), S(-43,-5), S(-83,-57), S(-70,-30), S(-60,-8), S(-46,19), S(-41,22), S(-31,20), S(-19,22), S(-14,27), S(-3,15), S(20,15), S(9,22), S(19,17), S(188,-98), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-364,-136), S(-364,-136), S(83,-112), S(-51,-471), S(-94,-115), S(-81,25), S(-71,71), S(-59,53), S(-61,154), S(-59,133), S(-58,159), S(-47,127), S(-49,167), S(-41,131), S(-43,150), S(-33,122), S(-30,111), S(-24,121), S(2,37), S(-27,80), S(-32,127), S(76,-67), S(-8,62), S(253,-154), S(100,-14), S(387,-305), S(-6,97), S(700,-416)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-36,-19), S(-27,-10), S(-21,17), S(20,20), S(49,17), S(177,118), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(12,31), S(10,15), S(3,5), S(1,1), S(-2,16), S(7,15), S(-18,4)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-58,-11), S(-11,9), S(-5,18), S(20,15), S(13,21), S(22,25), S(-8,24)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(5,-20), S(19,-41), S(1,-80), S(-185,-137)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(2,11), S(-7,36), S(14,63), S(9,101)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(12,-1), S(17,10), S(23,18), S(43,41), S(121,110), S(204,127), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(22,11), S(12,15), S(23,21), S(27,48), S(132,30), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(26,-4);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(14,5);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(138,-53);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(36,-13);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(12,30), S(13,18), S(11,11), S(4,84)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-20,18), S(-26,-4), S(-33,-1), S(-29,6), S(-22,-1), S(2,-7), S(28,-11), S(55,-12), S(97,-44), S(133,-77), S(233,-124), S(159,-38), S(225,-68), S(215,-81)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(15,-9), S(-34,34), S(-59,27), S(-43,21), S(-13,-5), S(11,-11), S(-84,-6), S(0,0)],
    [S(33,-6), S(-19,-1), S(-17,7), S(0,-3), S(15,-3), S(7,-17), S(-67,22), S(0,0)],
    [S(25,0), S(-14,4), S(-2,-11), S(13,-12), S(-3,-1), S(-30,-7), S(-80,16), S(0,0)],
    [S(39,4), S(6,-10), S(10,7), S(5,-2), S(7,8), S(20,-1), S(162,-31), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(-24,31), S(26,-142), S(0,-40), S(-34,-3), S(-42,21), S(-53,37), S(-46,23), S(0,0)],
    [S(3,5), S(53,-101), S(58,-45), S(-7,-16), S(-3,-5), S(-36,9), S(-3,8), S(0,0)],
    [S(8,11), S(77,-105), S(87,-52), S(7,-11), S(7,-8), S(-4,18), S(-15,11), S(0,0)],
    [S(26,4), S(95,-101), S(36,-35), S(20,9), S(19,3), S(10,8), S(3,-2), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-37,-24), S(83,32), S(65,51), S(50,50), S(38,104), S(0,0)],
    [S(-28,-7), S(224,134), S(285,133), S(315,318), S(372,1016), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(21,38), S(-18,-15), S(47,48), S(66,15), S(99,-131), S(0,0)],
        [S(-7,14), S(-21,-43), S(49,43), S(74,42), S(68,14), S(0,0)],
    ],
    [
        [S(30,53), S(53,71), S(175,145), S(152,326), S(521,895), S(0,0)],
        [S(-11,15), S(-14,-28), S(44,71), S(122,164), S(356,769), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(13,42), S(58,45), S(-26,10), S(73,51), S(103,78), S(0,0)],
        [S(8,5), S(31,40), S(-47,-26), S(71,67), S(66,175), S(0,0)],
    ],
    [
        [S(39,53), S(158,101), S(131,33), S(224,322), S(461,1125), S(0,0)],
        [S(4,4), S(26,42), S(-38,-39), S(104,157), S(389,914), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(7,62), S(44,74), S(63,64), S(-142,-101), S(139,-33), S(0,0)],
        [S(2,26), S(16,34), S(36,30), S(-140,-105), S(133,-2), S(0,0)],
    ],
    [
        [S(23,72), S(129,144), S(151,198), S(-7,225), S(512,1069), S(0,0)],
        [S(0,11), S(9,13), S(28,9), S(-138,-110), S(450,333), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(10,30), S(51,32), S(48,110), S(71,-9), S(-50,-155), S(0,0)],
        [S(-2,20), S(5,-3), S(-3,56), S(12,-5), S(-86,-164), S(0,0)],
    ],
    [
        [S(33,59), S(128,113), S(227,89), S(335,230), S(259,733), S(0,0)],
        [S(-2,16), S(-4,12), S(-7,47), S(-8,28), S(-68,-175), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(12,12), S(24,5)];
#[rustfmt::skip]
const TEMPO: i32 = 29;

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
