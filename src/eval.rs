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
const MATERIAL: [ScorePair; 6] = [S(84,122), S(317,232), S(408,274), S(488,481), S(975,960), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(96,56), S(-3,72), S(79,58), S(87,33), S(70,52), S(0,65), S(79,8), S(10,67),
        S(7,30), S(-14,40), S(7,14), S(31,3), S(27,0), S(18,1), S(-0,26), S(4,33),
        S(-33,14), S(-25,4), S(-16,-12), S(2,-34), S(7,-33), S(-3,-24), S(-10,-11), S(-11,-16),
        S(-45,5), S(-43,10), S(-12,-26), S(-3,-31), S(-4,-43), S(6,-36), S(-17,-25), S(-24,-17),
        S(-46,-6), S(-26,-18), S(-19,-25), S(-25,-19), S(-9,-32), S(-22,-24), S(13,-42), S(-19,-24),
        S(-45,13), S(-15,-2), S(-20,-10), S(-28,-3), S(-22,-21), S(11,-14), S(24,-32), S(-16,-25),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-166,-18), S(26,-15), S(-61,-35), S(-6,-36), S(-46,-47), S(-179,8), S(129,-75), S(-44,-113),
        S(-47,23), S(11,2), S(-3,-21), S(27,-7), S(31,5), S(-5,-6), S(-26,-1), S(-67,10),
        S(43,-19), S(-36,9), S(-0,22), S(38,28), S(27,18), S(-13,56), S(11,15), S(5,-8),
        S(47,-11), S(19,0), S(24,17), S(45,36), S(17,35), S(39,20), S(16,25), S(41,15),
        S(11,-20), S(32,-28), S(27,38), S(13,50), S(24,56), S(32,34), S(19,16), S(28,23),
        S(-14,-0), S(8,-14), S(-10,47), S(10,29), S(5,52), S(-1,34), S(25,-10), S(-4,13),
        S(-65,-64), S(7,-23), S(-5,5), S(-5,26), S(7,-3), S(8,1), S(-13,20), S(33,-35),
        S(-55,-86), S(-17,21), S(-33,7), S(-11,-14), S(16,-16), S(8,25), S(-11,-12), S(36,-106),
    ],
    [
        S(-16,-17), S(-68,35), S(-41,-8), S(-26,-26), S(-109,9), S(-138,11), S(-150,57), S(-15,67),
        S(24,-61), S(10,-3), S(-64,16), S(3,-20), S(-55,3), S(3,-2), S(-76,22), S(-46,0),
        S(41,-57), S(54,-29), S(22,6), S(24,-30), S(46,2), S(-3,40), S(35,-8), S(23,10),
        S(49,-24), S(15,-3), S(30,7), S(47,12), S(17,31), S(-0,18), S(9,11), S(45,-13),
        S(40,-9), S(11,-18), S(28,-4), S(23,36), S(19,30), S(8,17), S(35,13), S(5,19),
        S(12,6), S(60,-23), S(4,-1), S(11,8), S(1,14), S(33,3), S(27,-35), S(12,12),
        S(42,-14), S(1,2), S(23,-16), S(-7,-6), S(12,-15), S(13,-34), S(23,-29), S(23,-39),
        S(-30,40), S(13,-18), S(0,0), S(-50,9), S(-13,-39), S(-14,-4), S(-26,-6), S(-32,45),
    ],
    [
        S(-1,43), S(122,-24), S(167,-62), S(87,-50), S(64,-33), S(53,-21), S(-46,46), S(-10,42),
        S(9,27), S(50,-15), S(82,-18), S(65,-10), S(116,-30), S(64,-10), S(7,7), S(-51,48),
        S(-37,29), S(6,-0), S(-0,9), S(52,-9), S(0,6), S(-19,9), S(32,-0), S(-32,44),
        S(4,10), S(-26,14), S(-5,25), S(46,-12), S(1,-2), S(-16,21), S(-6,35), S(12,-4),
        S(-28,14), S(-23,3), S(20,8), S(-47,29), S(2,-7), S(-26,5), S(8,-12), S(-43,10),
        S(-53,1), S(1,-24), S(-19,-7), S(-52,12), S(6,-19), S(-22,-23), S(-32,15), S(-53,-1),
        S(-62,-10), S(-31,-7), S(-15,-15), S(-8,-33), S(-9,-35), S(-29,6), S(-18,-20), S(-82,-18),
        S(-45,11), S(-31,3), S(-12,2), S(-11,2), S(13,-16), S(-5,-5), S(-44,18), S(-37,-4),
    ],
    [
        S(-2,46), S(20,-3), S(-64,105), S(-4,38), S(58,-20), S(71,-32), S(-66,96), S(14,-9),
        S(-4,-27), S(-31,26), S(-57,69), S(-12,39), S(21,22), S(-40,113), S(-0,-35), S(24,-10),
        S(-17,47), S(-9,-1), S(-15,32), S(-51,114), S(-40,91), S(30,62), S(1,40), S(40,-34),
        S(19,-9), S(19,21), S(-33,107), S(-19,105), S(1,80), S(-19,47), S(20,-7), S(38,-49),
        S(12,-53), S(0,-52), S(17,19), S(-3,37), S(2,47), S(1,16), S(28,-2), S(28,-35),
        S(14,-70), S(7,-29), S(16,-19), S(4,-5), S(1,-21), S(-2,22), S(10,-16), S(19,-115),
        S(-37,84), S(-9,-35), S(18,-87), S(15,-55), S(1,-6), S(29,-91), S(14,-47), S(-10,-84),
        S(-6,22), S(16,-113), S(25,-72), S(13,-46), S(27,-109), S(6,-152), S(-56,-41), S(-89,44),
    ],
    [
        S(69,-132), S(-79,-34), S(266,-76), S(218,-16), S(-233,162), S(-349,177), S(-36,-3), S(12,-31),
        S(-416,58), S(-44,35), S(17,29), S(-349,144), S(-265,167), S(-428,131), S(-3,21), S(-133,-6),
        S(-43,32), S(-83,43), S(-172,74), S(-159,54), S(-64,28), S(-338,111), S(-345,141), S(-407,160),
        S(-71,14), S(149,2), S(126,4), S(183,-57), S(410,-84), S(153,-7), S(-7,33), S(-0,6),
        S(61,-55), S(200,-41), S(118,-25), S(143,-24), S(229,-49), S(125,-8), S(88,-25), S(-50,-5),
        S(98,-45), S(136,-57), S(42,-17), S(94,-26), S(23,-7), S(98,-25), S(111,-54), S(12,-25),
        S(111,-73), S(66,-48), S(75,-31), S(33,-18), S(28,-11), S(37,-13), S(93,-37), S(78,-52),
        S(1,-58), S(77,-43), S(62,-35), S(-14,-21), S(63,-67), S(26,-50), S(91,-57), S(63,-79),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-165,-290), S(-21,13), S(-7,28), S(8,38), S(20,50), S(26,48), S(33,54), S(44,56), S(63,3), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-55,-112), S(-31,-96), S(-51,-28), S(-37,16), S(-25,28), S(-23,52), S(-21,56), S(-15,49), S(-10,53), S(5,36), S(2,44), S(50,-18), S(17,21), S(195,-101), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-381,-382), S(-66,-5), S(-21,-45), S(-18,-2), S(-8,15), S(2,35), S(13,38), S(17,45), S(21,54), S(29,58), S(31,61), S(28,68), S(59,55), S(51,62), S(243,-61), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-302,-268), S(-302,-268), S(-667,-367), S(-115,460), S(-46,-105), S(-46,32), S(-43,82), S(-36,87), S(-33,107), S(-29,107), S(-31,131), S(-25,137), S(-21,137), S(-26,157), S(-23,142), S(-26,139), S(-26,141), S(-18,115), S(-26,102), S(0,92), S(-17,97), S(6,40), S(92,-40), S(243,-157), S(249,-167), S(319,-277), S(618,-369), S(330,-288)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-6,-48), S(-11,-20), S(-18,12), S(-3,34), S(6,54), S(107,125), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-22,54), S(-13,35), S(9,5), S(7,3), S(10,13), S(33,10), S(18,13)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(21,-43), S(23,-9), S(10,17), S(2,39), S(-9,54), S(4,39), S(-42,63)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(-23,-9), S(10,-51), S(18,-97), S(-144,-169)];
#[rustfmt::skip]
const PASSED_SAFE_ADV: [ScorePair; 4] = [S(-10,24), S(-17,60), S(-21,89), S(68,93)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,12), S(7,21), S(13,25), S(38,58), S(88,201), S(359,459), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(13,20), S(12,15), S(13,15), S(50,35), S(552,-138), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(27,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(19,15);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(92,-19);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(30,35);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(11,24), S(-6,27), S(16,20), S(-12,74)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-62,54), S(-53,1), S(-48,8), S(-36,8), S(-8,-11), S(25,-18), S(67,-46), S(111,-79), S(203,-109), S(193,-78), S(321,-228), S(348,-186), S(200,-163), S(314,-371)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-14,-66), S(79,38), S(61,58), S(58,60), S(16,161), S(0,0)],
    [S(-8,-53), S(203,176), S(230,219), S(241,445), S(432,1454), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,65), S(3,-130), S(50,33), S(55,40), S(34,179), S(0,0)],
        [S(-1,14), S(-17,-152), S(35,51), S(67,22), S(27,212), S(0,0)],
    ],
    [
        [S(24,64), S(50,-94), S(161,147), S(164,396), S(213,1318), S(0,0)],
        [S(-4,14), S(-9,-149), S(45,51), S(84,215), S(190,871), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,44), S(65,17), S(-19,7), S(45,63), S(102,95), S(0,0)],
        [S(4,9), S(28,35), S(-22,-7), S(35,105), S(66,153), S(0,0)],
    ],
    [
        [S(27,65), S(153,144), S(95,24), S(220,419), S(420,1248), S(0,0)],
        [S(-1,9), S(23,50), S(-29,-25), S(89,214), S(410,334), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(9,63), S(73,49), S(71,57), S(2,-242), S(116,42), S(0,0)],
        [S(-4,29), S(15,27), S(33,22), S(-1,-244), S(111,120), S(0,0)],
    ],
    [
        [S(9,87), S(111,195), S(139,210), S(135,180), S(542,1325), S(0,0)],
        [S(-5,13), S(24,7), S(33,6), S(-2,-265), S(266,751), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(17,-8), S(36,8), S(50,86), S(81,-6), S(-63,-202), S(0,0)],
        [S(-5,13), S(1,-5), S(17,5), S(4,-11), S(-126,-156), S(0,0)],
    ],
    [
        [S(25,69), S(127,33), S(185,123), S(299,225), S(535,754), S(0,0)],
        [S(-2,-1), S(-4,-7), S(-12,48), S(3,6), S(-103,-209), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,1), S(26,-3)];
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
