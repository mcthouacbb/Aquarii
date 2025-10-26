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
const MATERIAL: [ScorePair; 6] = [S(64,112), S(321,318), S(367,353), S(470,606), S(978,1044), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(60,56), S(31,70), S(33,40), S(49,36), S(61,35), S(10,36), S(7,54), S(52,54),
        S(1,18), S(0,20), S(2,2), S(18,-20), S(34,-23), S(-17,18), S(-6,23), S(-11,15),
        S(-14,7), S(-8,-2), S(-8,-15), S(-2,-26), S(1,-26), S(-3,-14), S(-0,-3), S(-13,3),
        S(-24,-7), S(-16,-7), S(-3,-25), S(-6,-29), S(-2,-30), S(-4,-26), S(-14,-8), S(-24,-12),
        S(-32,-12), S(-19,-9), S(-13,-21), S(-9,-19), S(-3,-20), S(-17,-17), S(1,-19), S(-31,-18),
        S(-25,-6), S(-8,-6), S(-4,-15), S(-9,-6), S(-5,-5), S(-1,-12), S(13,-14), S(-19,-15),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-126,-16), S(-63,13), S(-49,9), S(4,-4), S(41,-17), S(-122,27), S(-50,-18), S(-106,-46),
        S(-11,-12), S(-30,14), S(-4,11), S(23,22), S(27,-2), S(39,-18), S(-18,5), S(2,1),
        S(3,-1), S(25,-1), S(32,2), S(36,11), S(52,1), S(71,-11), S(42,0), S(34,-6),
        S(14,20), S(18,9), S(26,9), S(50,7), S(34,21), S(43,12), S(37,9), S(32,-2),
        S(4,12), S(22,3), S(20,12), S(22,22), S(25,14), S(32,3), S(36,1), S(23,8),
        S(-23,3), S(-14,-1), S(-9,3), S(4,12), S(8,7), S(-3,-5), S(11,-10), S(-13,-9),
        S(-15,-14), S(-30,5), S(-12,-16), S(-3,-1), S(-2,-8), S(1,-18), S(-4,-12), S(-4,-12),
        S(-39,-16), S(-28,-2), S(-25,-2), S(-10,-11), S(-12,-3), S(-6,-14), S(-20,-4), S(-42,2),
    ],
    [
        S(-38,29), S(-49,9), S(-49,17), S(-126,26), S(-48,12), S(-136,41), S(-87,13), S(-23,-19),
        S(-8,-10), S(-18,4), S(-21,8), S(-18,14), S(-21,2), S(-9,-2), S(-35,17), S(-13,-5),
        S(4,5), S(8,11), S(19,1), S(17,2), S(26,0), S(42,8), S(18,3), S(32,-4),
        S(-7,5), S(15,7), S(11,9), S(30,9), S(11,14), S(23,2), S(18,10), S(9,2),
        S(18,4), S(-0,9), S(20,2), S(17,2), S(28,-6), S(12,2), S(14,-0), S(14,-9),
        S(8,-4), S(16,-3), S(10,-5), S(15,-7), S(11,1), S(15,-3), S(14,-10), S(12,-5),
        S(27,-1), S(12,-12), S(16,-17), S(3,-12), S(10,-12), S(15,-19), S(21,-17), S(22,-11),
        S(26,-18), S(4,-9), S(2,-1), S(1,-14), S(3,-20), S(-5,2), S(28,-23), S(15,-24),
    ],
    [
        S(26,11), S(45,11), S(36,8), S(53,7), S(52,-2), S(50,2), S(49,4), S(39,7),
        S(7,23), S(-3,26), S(15,23), S(49,14), S(52,3), S(60,-3), S(23,11), S(29,11),
        S(-19,27), S(-1,14), S(11,9), S(10,9), S(47,-12), S(39,-1), S(22,0), S(-11,17),
        S(-23,20), S(-11,14), S(1,8), S(8,3), S(12,0), S(15,4), S(15,-3), S(-19,8),
        S(-39,7), S(-22,11), S(-10,1), S(-5,1), S(-1,-0), S(-0,-4), S(-6,-6), S(-22,-4),
        S(-46,-5), S(-29,-4), S(-28,-5), S(-11,-7), S(-8,-16), S(-22,-1), S(-9,-22), S(-40,-8),
        S(-54,-5), S(-35,-9), S(-17,-16), S(-15,-13), S(-10,-20), S(-11,-16), S(-30,-23), S(-68,-6),
        S(-40,-5), S(-23,-8), S(-9,-15), S(-3,-15), S(3,-20), S(-8,-17), S(-30,-9), S(-31,-16),
    ],
    [
        S(-16,29), S(4,8), S(28,13), S(56,-27), S(26,29), S(13,37), S(18,19), S(7,40),
        S(-12,10), S(-36,30), S(-27,44), S(-15,42), S(3,43), S(27,33), S(6,29), S(31,20),
        S(-10,0), S(-7,9), S(-19,28), S(1,35), S(-8,72), S(14,33), S(18,34), S(8,16),
        S(-2,-7), S(-5,24), S(-8,27), S(-20,60), S(-6,48), S(-4,41), S(3,37), S(4,17),
        S(-5,-12), S(-20,30), S(-12,28), S(-11,37), S(-8,29), S(3,11), S(3,13), S(-1,6),
        S(-13,-18), S(-7,-12), S(-8,3), S(-17,13), S(-8,3), S(-5,8), S(-7,5), S(-5,-19),
        S(5,-71), S(-3,-55), S(-2,-45), S(-2,-35), S(3,-34), S(3,-62), S(15,-96), S(15,-69),
        S(-11,-40), S(-2,-67), S(8,-81), S(-1,-33), S(4,-53), S(-2,-70), S(11,-100), S(13,-91),
    ],
    [
        S(165,-174), S(99,-13), S(71,20), S(-49,74), S(-30,53), S(122,-3), S(80,23), S(145,-195),
        S(30,-28), S(80,19), S(19,45), S(-13,54), S(74,65), S(45,62), S(-12,49), S(-16,-29),
        S(-10,31), S(4,67), S(28,12), S(-103,35), S(-72,23), S(-82,55), S(50,50), S(51,2),
        S(-29,7), S(-9,38), S(-63,34), S(-65,27), S(-83,18), S(-13,29), S(2,36), S(-73,10),
        S(-80,-5), S(-35,15), S(-49,19), S(-46,8), S(-37,11), S(-40,13), S(-52,24), S(-115,14),
        S(-9,-25), S(2,-5), S(-7,-2), S(2,-5), S(-2,-3), S(-8,-2), S(-24,7), S(-51,-7),
        S(21,-40), S(10,-14), S(24,-23), S(12,-19), S(7,-18), S(17,-18), S(10,-13), S(12,-34),
        S(7,-70), S(22,-43), S(36,-42), S(-8,-41), S(27,-55), S(-10,-35), S(14,-33), S(9,-58),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-107,-89), S(-24,-59), S(-11,-13), S(5,5), S(12,19), S(15,32), S(26,34), S(35,36), S(48,35), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-55,-108), S(-51,-71), S(-41,-30), S(-27,-4), S(-16,6), S(-9,18), S(-1,22), S(5,29), S(8,26), S(14,30), S(17,34), S(33,24), S(54,16), S(70,7), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-193,74), S(-8,-103), S(-35,-30), S(-29,-21), S(-17,-14), S(-4,-5), S(1,1), S(8,9), S(10,8), S(17,13), S(25,16), S(35,22), S(42,20), S(52,24), S(96,-14), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-295,-307), S(-295,-307), S(-234,-315), S(-28,-92), S(-90,51), S(-10,-29), S(-14,14), S(-3,25), S(-6,61), S(1,63), S(7,68), S(12,72), S(16,80), S(16,82), S(23,86), S(26,84), S(23,90), S(31,76), S(36,69), S(41,68), S(46,65), S(58,48), S(61,43), S(88,8), S(102,6), S(132,-40), S(156,-32), S(101,-37)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-15,-30), S(-19,-20), S(-7,4), S(13,20), S(25,57), S(80,120), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(10,28), S(-3,24), S(-6,9), S(-1,1), S(5,3), S(18,-3), S(8,4)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-63,-4), S(12,-9), S(6,12), S(6,20), S(5,24), S(7,27), S(-12,26)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-1,-15), S(-5,-32), S(-11,-55), S(-68,-98)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-7,13), S(-9,33), S(4,65), S(-1,72)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(7,3), S(15,11), S(15,28), S(37,64), S(116,139), S(-2,492), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(19,12), S(14,16), S(14,29), S(35,41), S(47,116), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(18,-2);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(19,19);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(59,-9);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(25,21);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(15,-0), S(7,-1), S(5,-8), S(-1,26)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-31,16), S(-37,9), S(-34,12), S(-29,12), S(-17,8), S(5,4), S(37,-7), S(67,-20), S(108,-43), S(143,-51), S(160,-62), S(228,-116), S(215,-25), S(443,-398)];
#[rustfmt::skip]
const PAWN_SHIELD: [[ScorePair; 8]; 4] = [
    [S(24,-6), S(-14,25), S(-30,18), S(-20,9), S(-6,-4), S(-13,-10), S(-26,-19), S(0,0)],
    [S(19,0), S(-22,10), S(-17,5), S(2,-5), S(10,-14), S(-8,-17), S(-14,-49), S(0,0)],
    [S(15,8), S(-16,4), S(2,-1), S(10,-4), S(16,-13), S(-5,-14), S(-20,-35), S(0,0)],
    [S(19,0), S(-11,2), S(-3,-0), S(1,-5), S(7,-6), S(34,-23), S(51,-57), S(0,0)],
];
#[rustfmt::skip]
const PAWN_STORM: [[ScorePair; 8]; 4] = [
    [S(-7,15), S(-90,-52), S(12,-27), S(-1,-11), S(-10,6), S(-17,15), S(-14,18), S(0,0)],
    [S(-0,10), S(2,-101), S(52,-48), S(0,-19), S(3,-11), S(-14,11), S(-11,13), S(0,0)],
    [S(-3,6), S(42,-76), S(77,-52), S(23,-25), S(1,-0), S(-6,3), S(-8,10), S(0,0)],
    [S(5,-4), S(51,-87), S(44,-35), S(18,-6), S(8,-3), S(-6,3), S(-1,-6), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-0,-33), S(66,35), S(59,48), S(50,37), S(56,12), S(0,0)],
    [S(-4,-26), S(67,128), S(56,163), S(81,166), S(62,138), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,30), S(-58,63), S(43,21), S(68,10), S(65,-7), S(0,0)],
        [S(-3,9), S(-58,48), S(28,20), S(65,9), S(36,36), S(0,0)],
    ],
    [
        [S(10,30), S(-101,60), S(57,23), S(111,60), S(68,-24), S(0,0)],
        [S(-7,4), S(-67,46), S(33,30), S(62,132), S(38,219), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(12,26), S(52,34), S(12,10), S(68,11), S(75,63), S(0,0)],
        [S(1,10), S(25,26), S(-1,-3), S(49,47), S(71,73), S(0,0)],
    ],
    [
        [S(16,30), S(44,68), S(-7,-8), S(104,100), S(127,12), S(0,0)],
        [S(-1,5), S(23,28), S(-2,-2), S(48,207), S(44,211), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(0,41), S(39,44), S(45,36), S(-15,-30), S(98,-35), S(0,0)],
        [S(-6,13), S(19,14), S(19,21), S(-14,-38), S(67,63), S(0,0)],
    ],
    [
        [S(2,41), S(56,67), S(75,76), S(-41,53), S(17,447), S(0,0)],
        [S(-8,2), S(15,9), S(27,7), S(-10,-51), S(190,108), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(11,-2), S(32,12), S(35,27), S(31,1), S(9,-21), S(0,0)],
        [S(-3,9), S(-1,6), S(-10,48), S(5,-13), S(-5,-7), S(0,0)],
    ],
    [
        [S(2,43), S(24,89), S(22,128), S(0,187), S(-72,130), S(0,0)],
        [S(-3,4), S(3,2), S(-4,41), S(5,-16), S(1,-13), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,4), S(21,6)];
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
