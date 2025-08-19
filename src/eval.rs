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
const MATERIAL: [ScorePair; 6] = [S(74,126), S(316,243), S(407,280), S(487,494), S(963,973), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(58,68), S(-24,66), S(27,74), S(57,12), S(33,47), S(-50,63), S(20,30), S(-34,75),
        S(11,38), S(-10,45), S(13,16), S(44,-7), S(39,-6), S(27,-1), S(7,27), S(12,34),
        S(-26,12), S(-19,6), S(-8,-13), S(11,-35), S(16,-33), S(4,-24), S(-3,-7), S(-2,-15),
        S(-35,1), S(-35,10), S(-6,-26), S(5,-32), S(5,-44), S(14,-35), S(-9,-23), S(-16,-18),
        S(-37,-8), S(-18,-16), S(-12,-24), S(-17,-19), S(0,-33), S(-13,-24), S(22,-41), S(-10,-25),
        S(-35,10), S(-7,-3), S(-12,-10), S(-20,-7), S(-12,-23), S(20,-18), S(32,-32), S(-6,-28),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-155,-22), S(15,-6), S(-78,-33), S(6,-57), S(-62,-60), S(-177,16), S(202,-122), S(-56,-96),
        S(-45,24), S(13,5), S(-8,-18), S(36,-10), S(16,7), S(-19,-1), S(-19,-8), S(-66,12),
        S(44,-32), S(-27,3), S(-5,26), S(31,28), S(32,14), S(-6,54), S(10,13), S(-4,-5),
        S(39,-10), S(19,2), S(21,20), S(45,35), S(18,32), S(41,21), S(17,22), S(41,16),
        S(11,-26), S(28,-24), S(25,40), S(14,46), S(22,59), S(31,34), S(18,7), S(27,22),
        S(-14,-1), S(4,-18), S(-10,45), S(6,35), S(5,52), S(-1,31), S(24,-10), S(-3,8),
        S(-63,-74), S(0,-17), S(-8,11), S(-5,30), S(7,-4), S(8,-7), S(-9,23), S(29,-23),
        S(-62,-22), S(-15,25), S(-28,21), S(-11,-19), S(21,-18), S(7,29), S(-8,-19), S(32,-103),
    ],
    [
        S(-18,-30), S(-58,34), S(-46,-3), S(-20,-30), S(-127,1), S(-143,8), S(-152,46), S(-11,54),
        S(13,-51), S(6,-1), S(-62,17), S(10,-24), S(-60,-6), S(7,-5), S(-78,16), S(-53,12),
        S(43,-58), S(47,-27), S(25,-1), S(34,-35), S(48,-2), S(5,31), S(27,-1), S(25,0),
        S(54,-31), S(15,-4), S(37,-3), S(47,6), S(16,29), S(-1,17), S(9,9), S(52,-20),
        S(40,-17), S(17,-21), S(31,-10), S(19,35), S(19,31), S(10,18), S(40,8), S(9,12),
        S(13,10), S(56,-20), S(2,-2), S(10,9), S(2,14), S(32,4), S(21,-33), S(13,12),
        S(36,2), S(2,6), S(25,-16), S(-5,-7), S(12,-15), S(17,-29), S(24,-27), S(23,-31),
        S(-35,65), S(13,2), S(2,-1), S(-60,22), S(-4,-38), S(-12,-5), S(-32,19), S(-31,54),
    ],
    [
        S(3,33), S(151,-40), S(167,-68), S(95,-56), S(84,-42), S(54,-22), S(-35,41), S(-13,35),
        S(16,18), S(39,-14), S(105,-32), S(75,-17), S(119,-31), S(52,-10), S(13,3), S(-42,37),
        S(-29,20), S(9,-7), S(2,1), S(41,-8), S(0,2), S(-10,0), S(20,-1), S(-29,32),
        S(1,6), S(-42,15), S(-1,17), S(38,-12), S(6,-5), S(-16,14), S(-5,27), S(15,-13),
        S(-32,10), S(-31,3), S(24,5), S(-47,30), S(10,-11), S(-25,4), S(1,-15), S(-44,8),
        S(-56,9), S(-12,-10), S(-23,1), S(-51,19), S(-6,-7), S(-23,-21), S(-36,17), S(-54,8),
        S(-66,10), S(-44,11), S(-28,4), S(-11,-21), S(-2,-35), S(-32,12), S(-23,-9), S(-81,-11),
        S(-47,20), S(-32,13), S(-16,10), S(-16,11), S(12,-11), S(-6,-1), S(-46,24), S(-38,1),
    ],
    [
        S(9,36), S(25,-2), S(-63,104), S(-7,26), S(43,-17), S(87,-40), S(-86,126), S(29,-41),
        S(-7,-29), S(-37,38), S(-59,72), S(-10,17), S(7,38), S(-39,118), S(-2,-22), S(26,-20),
        S(-10,15), S(-19,17), S(-16,26), S(-50,106), S(-26,65), S(44,53), S(-7,62), S(41,-37),
        S(25,-22), S(14,34), S(-32,92), S(-27,118), S(-2,83), S(-23,52), S(25,-5), S(39,-52),
        S(14,-61), S(-5,-41), S(16,25), S(-3,41), S(7,37), S(-2,24), S(25,3), S(22,-10),
        S(10,-72), S(4,-25), S(12,-12), S(2,2), S(5,-16), S(0,18), S(11,-21), S(17,-125),
        S(-44,104), S(-9,-33), S(17,-80), S(13,-46), S(3,-12), S(29,-92), S(14,-59), S(-12,-105),
        S(-12,32), S(20,-109), S(31,-90), S(12,-38), S(29,-107), S(2,-129), S(-59,-22), S(-61,9),
    ],
    [
        S(78,-131), S(-69,-24), S(274,-70), S(228,-16), S(-35,148), S(-287,175), S(-43,-9), S(47,-41),
        S(-285,21), S(15,13), S(-0,25), S(-347,146), S(-277,166), S(-394,130), S(-111,36), S(-25,-8),
        S(-1,22), S(-91,51), S(-167,69), S(-145,48), S(-79,30), S(-345,108), S(-360,139), S(-380,153),
        S(-103,15), S(100,7), S(83,8), S(167,-61), S(359,-80), S(133,-8), S(-17,31), S(4,-2),
        S(66,-69), S(190,-45), S(84,-26), S(139,-27), S(224,-52), S(107,-13), S(60,-26), S(-41,-12),
        S(97,-48), S(130,-54), S(47,-20), S(80,-22), S(18,-7), S(85,-22), S(99,-49), S(5,-26),
        S(99,-65), S(48,-36), S(66,-25), S(21,-11), S(17,-6), S(27,-10), S(80,-28), S(61,-44),
        S(-9,-51), S(65,-35), S(48,-29), S(-29,-10), S(50,-63), S(13,-44), S(77,-47), S(47,-69),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-174,-237), S(-20,7), S(-6,21), S(9,31), S(21,45), S(28,41), S(34,46), S(45,50), S(64,-4), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-74,-115), S(-27,-95), S(-52,-28), S(-37,14), S(-26,25), S(-23,49), S(-21,57), S(-15,50), S(-10,55), S(7,36), S(1,47), S(46,-16), S(11,22), S(219,-103), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-382,-295), S(-59,69), S(-25,-51), S(-20,-11), S(-10,9), S(1,27), S(10,34), S(15,35), S(21,39), S(30,45), S(37,43), S(29,49), S(61,36), S(60,39), S(231,-68), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-307,-276), S(-307,-276), S(-641,-316), S(-149,472), S(-46,-170), S(-42,56), S(-40,91), S(-33,98), S(-32,109), S(-26,106), S(-29,128), S(-22,141), S(-21,142), S(-22,154), S(-21,148), S(-23,141), S(-21,136), S(-15,116), S(-24,99), S(-2,89), S(1,77), S(16,32), S(90,-41), S(208,-140), S(212,-152), S(289,-267), S(576,-348), S(431,-350)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(5,-67), S(-7,-31), S(-25,15), S(-1,51), S(5,79), S(42,130), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-28,60), S(-15,39), S(2,10), S(2,4), S(6,16), S(31,12), S(18,14)];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0,0), S(-19,-62), S(21,-12), S(5,26), S(1,47), S(-11,63), S(1,52), S(-43,76)];
#[rustfmt::skip]
const PASSED_BLOCKED: [ScorePair; 4] = [S(0, 0); 4];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,11), S(8,17), S(14,22), S(45,59), S(53,240), S(562,698), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(13,20), S(12,15), S(14,16), S(53,35), S(506,-99), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(26,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(19,16);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(94,-19);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(32,31);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(10,24), S(-6,26), S(17,20), S(-10,69)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-64,54), S(-55,-0), S(-50,7), S(-39,7), S(-10,-13), S(19,-17), S(67,-47), S(117,-86), S(195,-105), S(184,-81), S(319,-248), S(345,-174), S(241,-169), S(292,-338)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-13,-77), S(77,41), S(63,49), S(70,37), S(13,157), S(0,0)],
    [S(-8,-63), S(203,175), S(236,210), S(250,436), S(454,1419), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(10,64), S(15,-140), S(49,37), S(59,36), S(43,147), S(0,0)],
        [S(-1,15), S(-9,-162), S(33,53), S(62,20), S(24,240), S(0,0)],
    ],
    [
        [S(26,65), S(51,-92), S(150,151), S(172,390), S(215,1308), S(0,0)],
        [S(-3,15), S(-1,-159), S(47,45), S(80,223), S(187,895), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(7,46), S(66,18), S(-22,5), S(47,64), S(97,82), S(0,0)],
        [S(5,8), S(28,36), S(-28,-10), S(44,89), S(60,192), S(0,0)],
    ],
    [
        [S(30,62), S(157,143), S(100,31), S(228,421), S(399,1182), S(0,0)],
        [S(-1,10), S(25,46), S(-33,-28), S(90,203), S(389,437), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(11,64), S(65,53), S(77,55), S(24,-227), S(102,36), S(0,0)],
        [S(-5,34), S(16,28), S(34,24), S(14,-222), S(105,117), S(0,0)],
    ],
    [
        [S(8,89), S(105,199), S(148,195), S(163,183), S(520,1319), S(0,0)],
        [S(-5,19), S(23,6), S(32,9), S(18,-252), S(299,695), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(14,-6), S(37,13), S(52,87), S(87,-25), S(-66,-167), S(0,0)],
        [S(-6,15), S(-1,1), S(11,24), S(7,-26), S(-127,-133), S(0,0)],
    ],
    [
        [S(23,76), S(122,56), S(189,119), S(300,192), S(482,745), S(0,0)],
        [S(-2,0), S(-7,-3), S(-13,47), S(2,-4), S(-100,-189), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,-1), S(26,-6)];
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

fn evaluate_pawns<Params: EvalValues>(board: &Board, color: Color) -> Params::ScorePairType {
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

    eval += evaluate_pawns::<Params>(board, stm) - evaluate_pawns::<Params>(board, !stm);

    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount()) as i32;

    (eval.mg() * phase.min(24) + eval.eg() * (24 - phase.min(24))) / 24 + Params::tempo()
}

pub fn eval(board: &Board) -> i32 {
    eval_impl::<EvalParams>(board)
}
