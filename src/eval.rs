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
const MATERIAL: [ScorePair; 6] = [S(229,348), S(914,1042), S(1050,1178), S(1282,2085), S(2658,3741), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(201,213), S(227,151), S(85,189), S(209,17), S(-2,116), S(20,60), S(-238,259), S(-290,335),
        S(32,118), S(36,131), S(91,-17), S(124,-136), S(189,-167), S(230,-75), S(229,42), S(108,76),
        S(-69,66), S(-47,12), S(-48,-45), S(-23,-111), S(28,-97), S(34,-81), S(33,-25), S(-4,-19),
        S(-95,13), S(-80,1), S(-66,-52), S(-37,-85), S(-30,-75), S(-22,-66), S(-17,-39), S(-46,-57),
        S(-124,-17), S(-100,-33), S(-98,-55), S(-83,-44), S(-48,-52), S(-64,-48), S(1,-68), S(-61,-75),
        S(-88,8), S(-62,-4), S(-46,-38), S(-57,-38), S(-15,-29), S(43,-41), S(69,-55), S(-30,-63),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-298,-103), S(-434,-50), S(-134,7), S(13,-40), S(-69,10), S(-93,-49), S(-183,0), S(-180,-228),
        S(-82,36), S(7,64), S(76,-43), S(105,-53), S(135,-76), S(105,-4), S(124,-12), S(90,15),
        S(-63,74), S(88,-19), S(123,-6), S(161,-19), S(119,-3), S(209,-69), S(75,-4), S(-39,49),
        S(32,49), S(30,26), S(78,45), S(153,35), S(101,24), S(105,33), S(52,20), S(90,28),
        S(-23,72), S(31,8), S(36,50), S(70,30), S(44,65), S(46,20), S(41,18), S(12,45),
        S(-43,-5), S(-26,-14), S(-16,-19), S(27,-3), S(43,27), S(-7,-50), S(23,-31), S(-23,13),
        S(-105,42), S(-60,47), S(-31,-38), S(1,-17), S(-7,-18), S(16,-41), S(-2,-43), S(-10,15),
        S(-213,39), S(-52,-21), S(-64,3), S(-48,18), S(-20,-8), S(-21,-28), S(-41,37), S(-74,51),
    ],
    [
        S(102,26), S(-273,74), S(59,-15), S(-236,20), S(-149,-32), S(-270,4), S(-125,44), S(-102,22),
        S(-41,14), S(-4,-6), S(-24,-57), S(-10,-6), S(-12,-53), S(-31,-48), S(-74,37), S(-26,-1),
        S(-10,38), S(59,-4), S(35,-31), S(65,-49), S(92,-52), S(77,-19), S(89,-9), S(38,17),
        S(-55,13), S(-19,3), S(12,23), S(82,27), S(16,19), S(33,7), S(-46,25), S(-49,40),
        S(-35,16), S(-46,24), S(-8,-0), S(37,-6), S(25,-1), S(-15,-23), S(-24,27), S(-5,-37),
        S(16,15), S(29,-3), S(15,-8), S(25,-18), S(31,8), S(35,-18), S(24,2), S(36,-1),
        S(59,-7), S(24,-53), S(51,-36), S(1,1), S(33,-26), S(56,-31), S(74,-41), S(45,38),
        S(19,13), S(40,68), S(19,20), S(22,-2), S(44,-7), S(9,27), S(86,-23), S(71,10),
    ],
    [
        S(-41,70), S(75,8), S(-3,85), S(76,33), S(50,33), S(28,73), S(260,-26), S(215,-23),
        S(-7,38), S(-11,59), S(30,71), S(123,16), S(43,15), S(80,36), S(90,-6), S(57,35),
        S(-30,31), S(65,2), S(14,21), S(29,30), S(90,-26), S(9,8), S(164,-2), S(23,28),
        S(-52,32), S(59,-4), S(-10,36), S(22,21), S(41,-46), S(9,-19), S(11,-7), S(4,-19),
        S(-85,0), S(-66,8), S(-30,31), S(-20,20), S(-9,0), S(-88,4), S(-49,-7), S(-90,28),
        S(-84,5), S(-78,8), S(-71,12), S(-39,-14), S(-46,2), S(-55,-51), S(-9,-70), S(-37,-69),
        S(-128,-0), S(-51,-33), S(-38,-17), S(-41,-21), S(-15,-49), S(-15,-59), S(-17,-42), S(-79,-37),
        S(-73,-16), S(-49,-13), S(-21,-10), S(3,-19), S(15,-52), S(-39,-30), S(-23,-59), S(-85,-53),
    ],
    [
        S(-25,40), S(-29,-100), S(-112,69), S(13,59), S(31,22), S(-3,89), S(405,-249), S(3,47),
        S(1,12), S(-24,-31), S(-38,138), S(-51,125), S(-91,228), S(36,31), S(-0,-14), S(100,35),
        S(16,-25), S(-27,25), S(-19,48), S(-74,79), S(5,67), S(48,52), S(63,-39), S(-33,129),
        S(-50,19), S(5,5), S(-37,47), S(-19,88), S(-0,54), S(-42,109), S(-38,45), S(-17,46),
        S(-2,-59), S(-33,26), S(-39,83), S(-57,168), S(-54,129), S(-56,85), S(-1,-5), S(-7,69),
        S(-12,-27), S(-12,5), S(-23,52), S(-31,52), S(-41,97), S(8,-34), S(9,-23), S(22,-15),
        S(30,-119), S(-16,-66), S(11,-95), S(24,-71), S(9,-21), S(34,-105), S(13,-175), S(97,-286),
        S(-8,-60), S(14,-114), S(26,-131), S(29,-58), S(32,-110), S(-26,-85), S(-6,-180), S(68,-177),
    ],
    [
        S(-11,-283), S(372,-229), S(294,-144), S(-109,-34), S(-87,2), S(-108,-32), S(-0,48), S(280,-436),
        S(-390,4), S(-259,68), S(-135,73), S(343,12), S(334,25), S(-52,131), S(275,64), S(21,-22),
        S(-251,33), S(82,45), S(179,79), S(33,156), S(292,154), S(453,125), S(179,156), S(30,26),
        S(98,-58), S(52,71), S(-123,126), S(-174,156), S(-139,180), S(-183,184), S(10,116), S(-301,26),
        S(-309,-11), S(-88,35), S(-120,101), S(-204,146), S(-229,167), S(-73,104), S(-158,73), S(-371,29),
        S(-7,-89), S(23,-27), S(-62,47), S(-90,77), S(-22,79), S(-49,50), S(19,-25), S(-148,-50),
        S(134,-120), S(89,-60), S(64,-10), S(-37,22), S(-46,38), S(13,-2), S(131,-72), S(107,-149),
        S(118,-286), S(177,-168), S(99,-106), S(-133,-30), S(15,-91), S(-48,-73), S(110,-154), S(90,-271),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-190,-879), S(-91,-83), S(-36,-4), S(-6,88), S(21,113), S(32,166), S(59,190), S(91,211), S(119,197), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-41,-501), S(-152,-217), S(-85,-149), S(-65,-62), S(-30,-7), S(-11,25), S(1,78), S(19,94), S(28,118), S(49,120), S(34,151), S(115,115), S(59,154), S(80,81), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-168,-801), S(-166,-21), S(-64,-90), S(-45,-23), S(-27,5), S(-10,25), S(-2,52), S(12,67), S(23,78), S(47,87), S(55,99), S(77,115), S(95,125), S(100,140), S(72,142), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-555,-806), S(-555,-806), S(-915,-941), S(-192,-287), S(-210,-169), S(-69,-127), S(-73,103), S(-59,98), S(-56,181), S(-34,192), S(-20,193), S(-7,223), S(13,221), S(36,217), S(30,246), S(36,269), S(56,245), S(45,266), S(58,270), S(58,278), S(62,266), S(145,173), S(213,150), S(328,-5), S(231,88), S(787,-319), S(418,-93), S(230,-126)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-25,26), S(-22,47), S(-17,118), S(31,208), S(-22,413), S(87,476), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(19,26), S(52,51), S(76,92), S(148,249), S(428,651), S(318,1506), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(72,73), S(45,56), S(33,80), S(41,150), S(433,270), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(63,26);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(99,72);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(227,-0);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(129,47);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(18,-11), S(8,-17), S(18,-52), S(-24,41)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-120,46), S(-112,42), S(-94,27), S(-74,59), S(-27,56), S(50,27), S(157,13), S(301,-57), S(464,-141), S(607,-198), S(641,-150), S(957,-165), S(602,370), S(384,185)];
#[rustfmt::skip]
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(-18,41), S(89,40), S(89,119), S(180,19), S(206,-200), S(0,0)],
    [S(-19,26), S(29,10), S(87,106), S(150,113), S(81,94), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(-16,74), S(75,149), S(11,10), S(257,47), S(236,126), S(0,0)],
    [S(8,57), S(57,130), S(-11,-10), S(156,157), S(214,487), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(-43,86), S(40,55), S(48,47), S(-56,-12), S(275,-56), S(0,0)],
    [S(1,41), S(40,72), S(80,83), S(-83,-53), S(195,228), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(-13,28), S(7,-11), S(5,57), S(73,-80), S(37,-55), S(0,0)],
    [S(-5,23), S(-9,52), S(-22,195), S(6,17), S(-47,46), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 80;

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
