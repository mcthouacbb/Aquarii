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
const MATERIAL: [ScorePair; 6] = [S(85,119), S(326,228), S(412,273), S(493,477), S(996,951), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(51,52), S(35,50), S(66,57), S(127,36), S(61,38), S(72,53), S(25,45), S(68,63),
        S(12,36), S(3,36), S(18,7), S(37,1), S(27,5), S(3,12), S(-27,29), S(-13,19),
        S(-14,1), S(-11,4), S(2,-19), S(4,-30), S(2,-31), S(-27,-13), S(-30,-11), S(-38,-9),
        S(-19,-7), S(-16,-10), S(6,-31), S(-2,-40), S(-10,-29), S(-19,-26), S(-48,-5), S(-55,-9),
        S(-9,-18), S(23,-36), S(-15,-12), S(-10,-21), S(-26,-22), S(-30,-31), S(-38,-22), S(-62,-15),
        S(-8,-11), S(36,-17), S(17,-8), S(-20,-12), S(-32,-15), S(-30,-11), S(-26,-17), S(-59,-2),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-94,-49), S(125,-129), S(-164,-21), S(-68,13), S(19,-104), S(-90,-1), S(-0,27), S(-133,-65),
        S(-45,21), S(-2,-15), S(16,-9), S(65,-14), S(-3,14), S(-20,-7), S(17,-7), S(-49,14),
        S(27,-14), S(16,-9), S(22,45), S(36,16), S(33,39), S(-7,30), S(-29,24), S(37,-20),
        S(45,-5), S(22,11), S(48,3), S(26,39), S(46,29), S(13,42), S(14,20), S(41,10),
        S(27,32), S(23,21), S(33,25), S(30,49), S(9,55), S(22,51), S(29,-29), S(7,-30),
        S(-2,2), S(26,-16), S(-3,43), S(-2,56), S(1,48), S(-18,44), S(-8,0), S(-22,1),
        S(38,-76), S(-16,7), S(9,10), S(4,-7), S(-14,33), S(-15,-4), S(10,-3), S(-77,-25),
        S(34,-110), S(-11,4), S(-6,37), S(30,-30), S(-25,-13), S(-1,-15), S(-22,15), S(-53,-104),
    ],
    [
        S(-16,12), S(-154,53), S(-133,11), S(-87,-7), S(-57,-31), S(-24,-7), S(-99,54), S(-19,-7),
        S(-36,-44), S(-55,16), S(-12,-5), S(-70,9), S(13,-25), S(-51,28), S(3,15), S(-24,1),
        S(22,7), S(36,-9), S(2,36), S(61,-8), S(24,-30), S(26,7), S(39,-22), S(34,-49),
        S(54,-7), S(11,19), S(23,17), S(19,23), S(52,19), S(30,-6), S(14,-8), S(55,-38),
        S(10,1), S(42,-4), S(11,4), S(24,38), S(17,29), S(28,10), S(11,-3), S(30,10),
        S(20,14), S(27,-7), S(33,2), S(2,13), S(10,8), S(-1,13), S(54,-42), S(6,10),
        S(28,-53), S(27,-17), S(24,-28), S(12,-16), S(-10,-6), S(18,-18), S(-0,-7), S(37,9),
        S(-40,57), S(-36,-34), S(-14,4), S(-2,-19), S(-63,0), S(-2,-7), S(17,-1), S(-3,14),
    ],
    [
        S(-7,36), S(-28,25), S(65,-35), S(132,-81), S(53,-19), S(156,-52), S(82,-3), S(1,45),
        S(-40,44), S(-4,24), S(62,-10), S(143,-54), S(51,4), S(93,-26), S(61,-24), S(2,30),
        S(-46,33), S(34,-14), S(-7,7), S(50,-6), S(18,4), S(7,7), S(14,5), S(-27,39),
        S(16,-0), S(-10,28), S(-16,13), S(36,-20), S(11,8), S(-8,34), S(-32,23), S(-14,15),
        S(-51,12), S(10,-6), S(-38,18), S(-1,-3), S(-39,21), S(21,-1), S(-25,4), S(-29,11),
        S(-61,6), S(-29,30), S(-17,-27), S(-10,-12), S(-38,8), S(-29,2), S(-21,-18), S(-48,1),
        S(-87,-11), S(-26,-7), S(-26,9), S(2,-41), S(-11,-29), S(-23,-17), S(-29,-13), S(-59,-15),
        S(-44,-10), S(-57,14), S(-6,-7), S(5,-14), S(-7,-1), S(-12,-2), S(-27,5), S(-38,10),
    ],
    [
        S(34,-48), S(-72,111), S(-65,112), S(37,44), S(43,-46), S(39,17), S(0,8), S(-24,113),
        S(13,-3), S(13,-39), S(-5,84), S(15,23), S(-21,36), S(-25,40), S(-41,49), S(3,-10),
        S(35,-15), S(12,69), S(40,46), S(-31,89), S(-36,93), S(-10,52), S(-2,-25), S(-6,32),
        S(53,-67), S(25,-9), S(-11,42), S(7,91), S(-19,97), S(-35,102), S(12,30), S(3,8),
        S(13,-16), S(26,1), S(-0,22), S(7,63), S(-4,13), S(14,14), S(4,-31), S(15,-64),
        S(27,-147), S(13,-21), S(5,14), S(3,-13), S(3,-20), S(2,1), S(2,-17), S(-5,-59),
        S(-17,-94), S(27,-79), S(17,-75), S(8,-20), S(4,-52), S(18,-87), S(-14,-46), S(-49,80),
        S(-118,77), S(-41,-106), S(33,-170), S(6,-70), S(11,-34), S(24,-111), S(3,-92), S(-13,12),
    ],
    [
        S(81,-71), S(-137,8), S(-190,121), S(-101,107), S(0,0), S(0,0), S(0,0), S(0,0),
        S(-188,17), S(-15,32), S(-300,105), S(-318,165), S(0,0), S(0,0), S(0,0), S(0,0),
        S(-296,127), S(-274,119), S(-250,98), S(-127,47), S(0,0), S(0,0), S(0,0), S(0,0),
        S(-51,17), S(17,32), S(135,3), S(262,-63), S(0,0), S(0,0), S(0,0), S(0,0),
        S(-22,-22), S(85,-20), S(95,-10), S(160,-31), S(0,0), S(0,0), S(0,0), S(0,0),
        S(24,-32), S(88,-47), S(53,-15), S(22,-3), S(0,0), S(0,0), S(0,0), S(0,0),
        S(62,-50), S(65,-33), S(22,-11), S(4,-7), S(0,0), S(0,0), S(0,0), S(0,0),
        S(43,-75), S(63,-48), S(17,-32), S(39,-59), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-132,-302), S(-30,13), S(-12,37), S(2,41), S(16,54), S(23,48), S(30,53), S(41,55), S(60,2), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-78,-61), S(-41,-101), S(-53,-27), S(-38,14), S(-26,25), S(-23,47), S(-20,51), S(-15,48), S(-10,51), S(5,32), S(-1,42), S(41,-20), S(35,14), S(224,-115), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-394,-390), S(-53,-18), S(-22,-42), S(-18,1), S(-9,18), S(1,36), S(12,41), S(14,50), S(20,57), S(28,61), S(32,62), S(26,70), S(58,58), S(60,60), S(246,-61), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-309,-263), S(-309,-263), S(-660,-392), S(-133,490), S(-35,-217), S(-44,39), S(-43,71), S(-37,84), S(-37,105), S(-32,102), S(-37,129), S(-32,143), S(-27,136), S(-32,162), S(-29,152), S(-34,154), S(-29,140), S(-26,126), S(-26,103), S(-6,103), S(-21,101), S(23,35), S(86,-23), S(246,-152), S(227,-150), S(317,-277), S(695,-377), S(343,-262)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-19,-45), S(-16,-18), S(-22,17), S(6,34), S(14,52), S(115,119), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-25,53), S(-13,32), S(8,4), S(10,3), S(9,18), S(31,18), S(9,28)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-1,-43), S(9,-11), S(-7,19), S(4,39), S(-1,56), S(21,43), S(-18,69)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-22,-10), S(9,-46), S(17,-99), S(-149,-165)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-10,23), S(-14,60), S(-14,89), S(70,94)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(5,9), S(7,19), S(13,23), S(40,56), S(84,196), S(363,684), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(13,20), S(12,15), S(13,15), S(48,36), S(512,-123), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(28,-16);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(22,10);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(91,-21);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(30,32);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(10,25), S(-2,26), S(18,21), S(-5,69)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-51,52), S(-46,0), S(-44,8), S(-37,7), S(-12,-12), S(16,-17), S(55,-45), S(94,-73), S(183,-108), S(165,-85), S(296,-217), S(282,-159), S(200,-186), S(272,-332)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-20,-64), S(77,42), S(60,61), S(53,55), S(19,160), S(0,0)],
    [S(-15,-51), S(206,173), S(235,213), S(245,440), S(458,1405), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(6,61), S(10,-144), S(50,39), S(67,30), S(32,206), S(0,0)],
        [S(-2,15), S(-17,-163), S(34,50), S(63,25), S(25,187), S(0,0)],
    ],
    [
        [S(25,63), S(53,-104), S(154,166), S(160,397), S(187,1336), S(0,0)],
        [S(-5,15), S(-8,-147), S(45,41), S(80,220), S(193,783), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,47), S(58,30), S(-22,2), S(41,71), S(97,64), S(0,0)],
        [S(5,9), S(27,39), S(-25,-3), S(38,95), S(65,147), S(0,0)],
    ],
    [
        [S(26,67), S(169,125), S(99,22), S(212,429), S(404,1218), S(0,0)],
        [S(-1,10), S(24,45), S(-28,-24), S(93,215), S(409,360), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,61), S(72,52), S(79,50), S(-2,-249), S(100,60), S(0,0)],
        [S(-4,28), S(15,29), S(37,22), S(-15,-241), S(108,120), S(0,0)],
    ],
    [
        [S(7,83), S(113,187), S(161,192), S(138,163), S(517,1307), S(0,0)],
        [S(-4,11), S(23,7), S(32,7), S(-12,-271), S(297,702), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(15,-8), S(38,18), S(51,88), S(83,-6), S(-45,-227), S(0,0)],
        [S(-5,11), S(-2,4), S(13,12), S(8,-22), S(-104,-195), S(0,0)],
    ],
    [
        [S(22,75), S(118,68), S(189,106), S(305,219), S(437,957), S(0,0)],
        [S(-2,1), S(-6,-6), S(-11,41), S(4,-4), S(-81,-243), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,1), S(25,-3)];
#[rustfmt::skip]
const TEMPO: i32 = 26;

pub struct EvalParams {}

impl EvalValues for EvalParams {
    type ScoreType = i32;
    type ScorePairType = ScorePair;

    fn material(pt: PieceType) -> Self::ScorePairType {
        MATERIAL[pt as usize]
    }

    fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType {
        PSQT[pt as usize][sq.relative_sq(c).flip_vertical().value() as usize]
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
        let mirror_stm = board.king_sq(stm).file() >= 4;
        let mirror_nstm = board.king_sq(!stm).file() >= 4;
        let mut stm_bb = board.colored_pieces(Piece::new(stm, pt));
        let mut nstm_bb = board.colored_pieces(Piece::new(!stm, pt));

        while stm_bb.any() {
            let sq = if mirror_stm {
                stm_bb.poplsb().flip_horizontal()
            } else {
                stm_bb.poplsb()
            };
            eval += Params::material(pt) + Params::psqt(stm, pt, sq);
        }

        while nstm_bb.any() {
            let sq = if mirror_nstm {
                nstm_bb.poplsb().flip_horizontal()
            } else {
                nstm_bb.poplsb()
            };
            eval -= Params::material(pt) + Params::psqt(!stm, pt, sq);
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
