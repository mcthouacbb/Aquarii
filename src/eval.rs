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
    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn push_threat(stm: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(83,126), S(319,230), S(407,277), S(488,486), S(967,969), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(95,74), S(4,88), S(67,84), S(77,53), S(68,60), S(7,70), S(62,37), S(21,83),
        S(4,31), S(-15,38), S(11,12), S(36,-6), S(27,-9), S(19,-4), S(-0,23), S(6,28),
        S(-32,9), S(-25,2), S(-17,-16), S(3,-40), S(6,-33), S(-3,-26), S(-12,-12), S(-12,-18),
        S(-44,-1), S(-43,8), S(-12,-29), S(-3,-35), S(-5,-47), S(5,-39), S(-17,-26), S(-24,-21),
        S(-45,-12), S(-26,-20), S(-20,-26), S(-25,-22), S(-9,-35), S(-22,-25), S(14,-45), S(-19,-28),
        S(-43,7), S(-14,-4), S(-19,-11), S(-27,-7), S(-20,-26), S(12,-20), S(25,-34), S(-15,-30),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-162,-13), S(35,-21), S(-77,-31), S(-11,-42), S(-4,-71), S(-170,13), S(121,-83), S(-42,-99),
        S(-50,25), S(13,-6), S(-10,-16), S(38,-13), S(12,6), S(-27,1), S(-22,3), S(-53,12),
        S(41,-17), S(-28,1), S(7,9), S(36,26), S(29,13), S(-1,47), S(7,19), S(7,0),
        S(46,-22), S(19,2), S(25,15), S(43,33), S(20,28), S(42,17), S(17,21), S(40,15),
        S(10,-22), S(28,-24), S(23,37), S(13,44), S(24,51), S(33,25), S(24,6), S(26,26),
        S(-15,7), S(7,-21), S(-10,43), S(7,34), S(6,48), S(-0,30), S(24,-12), S(-1,13),
        S(-61,-69), S(2,-8), S(-8,7), S(-6,30), S(7,-6), S(6,1), S(-12,30), S(33,-16),
        S(-75,-77), S(-18,30), S(-38,23), S(-17,-4), S(22,-13), S(13,25), S(-8,-10), S(24,-105),
    ],
    [
        S(-18,-29), S(-102,47), S(-41,-6), S(-20,-35), S(-110,2), S(-142,20), S(-165,70), S(-21,60),
        S(16,-49), S(-4,10), S(-71,19), S(2,-22), S(-62,7), S(-1,-2), S(-77,26), S(-50,10),
        S(38,-55), S(47,-25), S(20,4), S(31,-34), S(44,-1), S(3,33), S(37,2), S(21,6),
        S(53,-29), S(20,-7), S(34,3), S(47,11), S(20,32), S(9,16), S(11,10), S(47,-17),
        S(41,-10), S(18,-23), S(31,-10), S(25,37), S(22,32), S(11,18), S(38,12), S(11,6),
        S(12,11), S(62,-23), S(2,-1), S(14,7), S(3,15), S(36,2), S(27,-33), S(16,11),
        S(43,-6), S(2,2), S(28,-14), S(-5,-8), S(14,-16), S(15,-30), S(26,-28), S(25,-43),
        S(-27,28), S(21,-17), S(3,-3), S(-55,18), S(-5,-36), S(-11,-7), S(-24,-2), S(-34,36),
    ],
    [
        S(5,33), S(132,-32), S(171,-67), S(92,-55), S(92,-47), S(39,-20), S(-62,49), S(-5,35),
        S(8,23), S(45,-14), S(96,-25), S(78,-18), S(124,-35), S(71,-20), S(14,2), S(-48,40),
        S(-35,28), S(14,-3), S(5,4), S(44,-9), S(13,3), S(-7,-1), S(27,-4), S(-34,42),
        S(6,9), S(-47,21), S(-8,23), S(40,-9), S(3,-3), S(-20,15), S(-2,30), S(16,-8),
        S(-27,7), S(-30,7), S(16,9), S(-51,31), S(3,-8), S(-20,-2), S(-2,-7), S(-40,12),
        S(-57,8), S(-11,-15), S(-23,-2), S(-53,20), S(-11,-6), S(-20,-20), S(-31,12), S(-53,6),
        S(-65,1), S(-38,-1), S(-18,-8), S(-9,-26), S(-12,-34), S(-29,10), S(-21,-11), S(-81,-15),
        S(-47,16), S(-34,14), S(-18,12), S(-14,7), S(14,-15), S(-5,-5), S(-46,20), S(-37,-4),
    ],
    [
        S(9,36), S(29,-23), S(-58,104), S(-9,32), S(88,-38), S(51,-17), S(-76,107), S(40,-47),
        S(1,-17), S(-33,27), S(-58,68), S(-10,27), S(7,26), S(-47,125), S(-7,-23), S(24,-17),
        S(-18,31), S(-20,24), S(-6,19), S(-43,94), S(-33,84), S(43,51), S(-4,53), S(43,-36),
        S(23,-21), S(22,19), S(-37,110), S(-19,109), S(5,74), S(-25,58), S(19,-9), S(38,-42),
        S(12,-55), S(-4,-34), S(16,22), S(-3,34), S(-5,56), S(-1,14), S(29,-0), S(27,-21),
        S(12,-83), S(6,-25), S(13,-22), S(1,-1), S(0,-17), S(-1,24), S(11,-27), S(14,-126),
        S(-40,80), S(-8,-35), S(16,-80), S(12,-50), S(2,-5), S(31,-100), S(18,-54), S(-11,-89),
        S(-7,20), S(8,-91), S(25,-99), S(13,-39), S(30,-100), S(-0,-120), S(-72,6), S(-82,31),
    ],
    [
        S(122,-138), S(-164,-16), S(157,-52), S(127,-12), S(-297,158), S(-338,176), S(-14,-13), S(75,-23),
        S(-303,51), S(-4,21), S(-25,32), S(-316,145), S(-283,159), S(-383,124), S(-10,25), S(-64,-13),
        S(-57,25), S(-43,44), S(-130,67), S(-151,55), S(-24,23), S(-319,105), S(-332,133), S(-390,161),
        S(-86,14), S(124,0), S(69,12), S(186,-60), S(397,-83), S(163,-8), S(-35,37), S(9,-0),
        S(90,-70), S(188,-42), S(116,-32), S(156,-27), S(218,-49), S(123,-13), S(69,-21), S(-43,-10),
        S(119,-56), S(142,-61), S(45,-18), S(78,-22), S(11,-1), S(98,-25), S(105,-51), S(5,-29),
        S(114,-70), S(63,-43), S(70,-27), S(31,-15), S(23,-9), S(33,-10), S(90,-33), S(72,-51),
        S(1,-49), S(75,-41), S(55,-29), S(-17,-18), S(57,-64), S(17,-43), S(85,-50), S(54,-72),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-147,-312), S(-26,13), S(-9,24), S(6,36), S(17,55), S(24,54), S(31,58), S(43,61), S(60,11), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-64,-111), S(-34,-99), S(-52,-26), S(-38,16), S(-26,30), S(-23,48), S(-21,54), S(-15,50), S(-11,52), S(4,37), S(-3,49), S(44,-13), S(33,15), S(205,-102), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-385,-306), S(-47,-21), S(-23,-48), S(-20,-8), S(-10,14), S(1,31), S(11,35), S(15,43), S(21,49), S(28,55), S(32,56), S(31,61), S(54,50), S(60,50), S(233,-62), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-299,-276), S(-299,-276), S(-647,-340), S(-148,566), S(-36,-172), S(-36,54), S(-32,70), S(-24,67), S(-24,96), S(-19,95), S(-21,116), S(-15,130), S(-13,130), S(-14,142), S(-13,135), S(-17,135), S(-16,131), S(-10,108), S(-15,98), S(5,85), S(-0,81), S(22,25), S(99,-45), S(220,-150), S(174,-134), S(295,-264), S(531,-307), S(352,-302)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-2,-67), S(-7,-37), S(-22,12), S(-5,52), S(-7,89), S(104,146), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-24,62), S(-14,41), S(6,11), S(5,7), S(10,18), S(32,17), S(17,18)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(7,-48), S(24,-12), S(7,27), S(3,49), S(-13,66), S(4,49), S(-37,73)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-25,-5), S(4,-33), S(16,-66), S(-135,-128)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(0, 0); 4];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(5,12), S(9,18), S(14,25), S(39,60), S(91,212), S(641,686), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(14,19), S(13,15), S(14,15), S(51,35), S(505,-126), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(25,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(20,16);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(94,-19);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(31,33);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(10,25), S(-6,27), S(15,21), S(-10,67)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-65,55), S(-56,1), S(-50,9), S(-39,8), S(-9,-13), S(20,-16), S(68,-51), S(110,-83), S(204,-112), S(201,-99), S(313,-249), S(324,-179), S(229,-180), S(255,-216)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-14,-77), S(75,40), S(63,57), S(50,38), S(12,171), S(0,0)],
    [S(-9,-63), S(206,168), S(233,209), S(251,441), S(488,1389), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,62), S(4,-164), S(48,42), S(61,42), S(41,141), S(0,0)],
        [S(-2,16), S(-19,-175), S(36,50), S(63,22), S(19,245), S(0,0)],
    ],
    [
        [S(24,66), S(48,-112), S(145,169), S(176,401), S(231,1266), S(0,0)],
        [S(-4,13), S(-12,-176), S(46,45), S(90,206), S(185,859), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,47), S(60,22), S(-22,5), S(45,66), S(100,64), S(0,0)],
        [S(4,8), S(27,37), S(-27,-9), S(41,97), S(66,139), S(0,0)],
    ],
    [
        [S(26,64), S(166,135), S(93,23), S(219,425), S(397,1173), S(0,0)],
        [S(-2,9), S(25,46), S(-32,-20), S(92,201), S(387,363), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,61), S(69,44), S(74,49), S(19,-234), S(104,51), S(0,0)],
        [S(-6,29), S(15,29), S(37,22), S(-6,-229), S(95,155), S(0,0)],
    ],
    [
        [S(5,85), S(108,202), S(156,191), S(146,184), S(476,1341), S(0,0)],
        [S(-6,13), S(23,6), S(32,8), S(-3,-249), S(280,723), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(15,-4), S(37,13), S(47,89), S(83,-11), S(-57,-256), S(0,0)],
        [S(-5,14), S(1,2), S(13,9), S(5,-12), S(-124,-203), S(0,0)],
    ],
    [
        [S(23,78), S(118,63), S(181,131), S(319,166), S(459,997), S(0,0)],
        [S(-3,2), S(-6,-6), S(-14,55), S(1,1), S(-100,-258), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,-3), S(26,-6)];
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

fn evaluate_kings<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let their_king = board.king_sq(!color);

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
