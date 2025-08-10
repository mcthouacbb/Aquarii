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
    const fn new(mg: i32, eg: i32) -> Self {
        Self((((eg as u32) << 16).wrapping_add(mg as u32)) as i32)
    }

    const fn mg(&self) -> i32 {
        self.0 as i16 as i32
    }

    const fn eg(&self) -> i32 {
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
    fn pawn_phalanx(rank: u8) -> Self::ScorePairType;
    fn defended_pawn(rank: u8) -> Self::ScorePairType;
    fn safe_knight_check() -> Self::ScorePairType;
    fn safe_bishop_check() -> Self::ScorePairType;
    fn safe_rook_check() -> Self::ScorePairType;
    fn safe_queen_check() -> Self::ScorePairType;
    fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType;
    fn king_attacks(attacks: u32) -> Self::ScorePairType;
    fn threat_by_pawn(pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(45,72), S(196,133), S(246,114), S(302,257), S(686,521), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(31,24), S(33,31), S(-6,80), S(1,14), S(64,-21), S(22,40), S(10,61), S(-68,78),
        S(18,-7), S(-32,27), S(-37,3), S(16,6), S(-9,-36), S(34,-32), S(-8,17), S(6,-0),
        S(-28,14), S(-6,18), S(10,2), S(9,-21), S(11,-31), S(-3,-4), S(2,-10), S(-28,16),
        S(-24,-6), S(-11,-17), S(2,-30), S(2,-21), S(-2,-14), S(6,-14), S(-7,-16), S(3,-17),
        S(-21,-0), S(-18,-21), S(-10,-7), S(-18,-6), S(-9,-11), S(-10,-5), S(10,-16), S(-8,-18),
        S(-16,-5), S(7,-2), S(10,-1), S(-6,7), S(2,-8), S(36,-2), S(27,-17), S(14,-25),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-43,-27), S(-0,-83), S(-2,-42), S(-73,31), S(-29,-7), S(-103,19), S(-79,-5), S(11,-106),
        S(-9,-27), S(-21,-19), S(27,-12), S(32,-10), S(-26,19), S(-7,-22), S(-48,45), S(3,3),
        S(-5,-42), S(38,-30), S(-10,28), S(16,-4), S(12,23), S(17,28), S(-2,-5), S(23,-31),
        S(52,-0), S(15,-2), S(24,-15), S(29,-16), S(35,16), S(16,12), S(37,10), S(14,-9),
        S(-8,-9), S(4,-19), S(22,5), S(20,18), S(16,32), S(25,13), S(19,-3), S(2,-6),
        S(-19,15), S(-8,-12), S(-2,44), S(21,16), S(26,-5), S(10,26), S(24,-21), S(20,-5),
        S(-23,-69), S(74,-67), S(11,-1), S(-1,35), S(12,5), S(20,-30), S(-25,23), S(12,-19),
        S(-131,136), S(-6,49), S(21,3), S(-18,31), S(-53,-3), S(31,2), S(3,54), S(-46,40),
    ],
    [
        S(-21,-25), S(-27,-29), S(-135,33), S(-126,15), S(11,-36), S(-127,9), S(-133,4), S(-35,42),
        S(-49,25), S(-12,4), S(38,-2), S(21,-33), S(-13,9), S(77,-15), S(-87,42), S(-70,10),
        S(-4,1), S(29,-41), S(-5,8), S(13,-7), S(39,-11), S(-38,60), S(12,-1), S(12,39),
        S(-11,-14), S(27,-4), S(-5,32), S(42,25), S(31,13), S(9,0), S(16,6), S(-15,-24),
        S(47,-54), S(22,20), S(26,14), S(5,42), S(13,25), S(24,11), S(3,7), S(35,-37),
        S(15,-2), S(42,-3), S(17,8), S(16,3), S(-0,11), S(6,1), S(32,-2), S(5,-33),
        S(19,-65), S(15,5), S(47,-23), S(1,-10), S(14,-2), S(-19,-13), S(28,-15), S(23,36),
        S(-23,26), S(64,8), S(1,-11), S(74,-15), S(10,-30), S(-2,-6), S(4,-65), S(-27,37),
    ],
    [
        S(11,11), S(88,-11), S(28,-3), S(120,-44), S(83,-14), S(68,-14), S(110,-45), S(14,8),
        S(-2,11), S(-27,19), S(7,-9), S(80,-21), S(81,-15), S(32,5), S(-3,4), S(44,-1),
        S(-57,24), S(15,5), S(-7,13), S(-1,6), S(9,-6), S(26,-14), S(29,-6), S(5,-4),
        S(6,5), S(-11,12), S(-13,15), S(-11,23), S(-15,10), S(-17,9), S(-32,4), S(-54,26),
        S(-36,1), S(-51,5), S(-35,16), S(27,-19), S(-6,-22), S(-6,8), S(-18,9), S(-33,-18),
        S(6,-19), S(-43,19), S(-9,9), S(-21,5), S(-9,-2), S(-1,-15), S(6,-30), S(-19,0),
        S(-59,1), S(-26,-10), S(-42,3), S(-11,-6), S(0,1), S(-32,16), S(1,-12), S(-56,9),
        S(-25,14), S(-22,13), S(-8,11), S(-1,-2), S(1,0), S(-3,5), S(-46,9), S(-30,-3),
    ],
    [
        S(3,8), S(54,-45), S(-106,111), S(41,19), S(42,25), S(134,-91), S(-9,4), S(-4,22),
        S(14,-14), S(-20,29), S(-11,18), S(23,-31), S(-5,49), S(15,13), S(-9,-42), S(23,-17),
        S(21,-54), S(-37,3), S(46,8), S(-29,44), S(-37,19), S(7,59), S(2,-0), S(-2,18),
        S(24,-85), S(15,-9), S(-2,9), S(-28,49), S(-11,83), S(-16,71), S(-7,22), S(11,23),
        S(9,22), S(-23,11), S(-35,85), S(-6,62), S(-19,84), S(-5,45), S(-13,16), S(-2,-15),
        S(-23,6), S(-1,6), S(-14,16), S(10,9), S(-23,92), S(-5,72), S(16,-31), S(19,-47),
        S(-26,-32), S(10,-56), S(-4,-34), S(16,-84), S(7,-21), S(-4,29), S(12,9), S(-9,-73),
        S(-27,-124), S(18,-148), S(-21,43), S(5,20), S(-6,-27), S(29,-93), S(-47,-27), S(18,-135),
    ],
    [
        S(293,-18), S(176,-22), S(275,-16), S(299,-22), S(-62,74), S(28,29), S(49,-20), S(-32,-45),
        S(-18,-61), S(25,24), S(289,-23), S(103,40), S(-29,73), S(-173,65), S(28,20), S(-33,-15),
        S(17,-18), S(-263,46), S(-216,52), S(-316,43), S(-41,15), S(-120,36), S(-310,79), S(-277,84),
        S(-145,29), S(-75,25), S(-154,42), S(125,-26), S(151,-39), S(140,-26), S(-46,33), S(218,-11),
        S(69,-10), S(-62,16), S(36,-1), S(35,4), S(67,-23), S(25,13), S(71,-4), S(-107,5),
        S(-124,6), S(-44,-19), S(45,-2), S(22,2), S(-59,16), S(-19,11), S(51,-18), S(-26,-24),
        S(-10,-32), S(-2,6), S(36,-39), S(25,-36), S(-29,2), S(-8,-4), S(29,-23), S(32,-46),
        S(9,-47), S(20,-13), S(20,-18), S(-38,-9), S(11,-40), S(-15,-30), S(22,-30), S(12,-61),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-50,-12), S(-14,-39), S(-9,-31), S(1,4), S(6,25), S(6,27), S(10,17), S(19,23), S(32,-14), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-43,-401), S(-58,-64), S(-30,13), S(-17,49), S(-13,32), S(-10,54), S(-6,56), S(-8,66), S(-8,61), S(8,50), S(-4,53), S(27,19), S(-23,53), S(185,-41), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-168,-98), S(73,-36), S(-20,-63), S(-12,-0), S(-9,-6), S(-1,17), S(-1,17), S(-3,22), S(1,25), S(6,25), S(2,29), S(4,32), S(17,22), S(2,34), S(109,-21), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-224,-180), S(-224,-180), S(-224,-180), S(-264,846), S(-175,610), S(-72,123), S(-82,65), S(-72,31), S(-79,83), S(-76,70), S(-76,71), S(-76,78), S(-74,82), S(-67,57), S(-67,49), S(-64,40), S(-66,37), S(-62,32), S(-64,14), S(-51,-4), S(-56,-16), S(-31,-17), S(-52,-13), S(126,-155), S(274,-246), S(385,-335), S(503,-349), S(1011,-612)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(5,14), S(0,34), S(9,37), S(21,41), S(14,90), S(25,84), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(-4,14), S(13,4), S(13,24), S(42,27), S(128,94), S(485,363), S(0,0)];
#[rustfmt::skip]
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(11,15), S(16,6), S(6,15), S(54,-0), S(419,-101), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(15,-11);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(14,5);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(56,-11);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(21,-3);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(12,18), S(1,8), S(-6,22), S(-7,41)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-28,33), S(-30,2), S(-25,2), S(-25,-1), S(-12,-4), S(7,-20), S(23,-21), S(52,-50), S(102,-76), S(155,-100), S(142,-73), S(255,-290), S(164,-217), S(408,-366)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(-28,-37), S(68,53), S(78,111), S(41,157), S(-22,627), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(3,48), S(-61,-124), S(57,41), S(60,50), S(39,295), S(0,0)],
    [S(-11,24), S(-72,-166), S(25,28), S(29,61), S(11,124), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(14,41), S(35,47), S(17,7), S(52,80), S(26,217), S(0,0)],
    [S(-2,14), S(8,42), S(-17,-7), S(32,74), S(13,183), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(9,51), S(53,45), S(75,44), S(-32,-80), S(59,130), S(0,0)],
    [S(-2,11), S(23,7), S(32,7), S(-45,-131), S(39,176), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(14,17), S(42,26), S(74,3), S(60,55), S(0,-0), S(0,0)],
    [S(-1,-1), S(-0,-3), S(-6,34), S(-11,14), S(-17,-27), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 24;

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

    fn threat_by_pawn(pt: PieceType) -> Self::ScorePairType {
        THREAT_BY_PAWN[pt as usize]
    }

    fn threat_by_knight(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_KNIGHT[defended as usize][pt as usize]
    }

    fn threat_by_bishop(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_BISHOP[defended as usize][pt as usize]
    }

    fn threat_by_rook(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_ROOK[defended as usize][pt as usize]
    }

    fn threat_by_queen(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_QUEEN[defended as usize][pt as usize]
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
    let mut eval = Params::ScorePairType::default();

    let defended_bb = eval_data.attacked_by_2[!color as usize]
        | eval_data.attacked_by[!color as usize][PieceType::Pawn as usize]
        | (eval_data.attacked[!color as usize] & !eval_data.attacked_by_2[color as usize]);

    let mut pawn_threats =
        eval_data.attacked_by[color as usize][PieceType::Pawn as usize] & board.colors(!color);
    while pawn_threats.any() {
        let threatened = board.piece_at(pawn_threats.poplsb()).unwrap().piece_type();
        eval += Params::threat_by_pawn(threatened);
    }

    let mut knight_threats =
        eval_data.attacked_by[color as usize][PieceType::Knight as usize] & board.colors(!color);
    while knight_threats.any() {
        let threat = knight_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_knight(threatened, defended);
    }

    let mut bishop_threats =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & board.colors(!color);
    while bishop_threats.any() {
        let threat = bishop_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_bishop(threatened, defended);
    }

    let mut rook_threats =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & board.colors(!color);
    while rook_threats.any() {
        let threat = rook_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_rook(threatened, defended);
    }

    let mut queen_threats =
        eval_data.attacked_by[color as usize][PieceType::Queen as usize] & board.colors(!color);
    while queen_threats.any() {
        let threat = queen_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_queen(threatened, defended);
    }

    eval
}

fn evaluate_pawns<Params: EvalValues>(board: &Board, color: Color) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();
    let our_pawns = board.colored_pieces(Piece::new(color, PieceType::Pawn));
    let their_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));

    let mut tmp = our_pawns;
    while tmp.any() {
        let sq = tmp.poplsb();
        let relative_rank = sq.relative_sq(color).rank();
        let stoppers = their_pawns & attacks::passed_pawn_span(color, sq);
        if stoppers.empty() {
            eval += Params::passed_pawn(relative_rank);
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
