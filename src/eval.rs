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
const MATERIAL: [ScorePair; 6] = [S(149,208), S(641,285), S(787,431), S(1103,641), S(1830,1460), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(368,-9), S(231,-29), S(13,147), S(-13,121), S(256,-44), S(39,139), S(-475,311), S(108,139),
        S(-112,26), S(59,95), S(-161,41), S(-111,-60), S(-10,-61), S(-31,-22), S(60,-20), S(-39,-24),
        S(30,-10), S(27,-45), S(-3,34), S(25,-84), S(8,-67), S(-96,-43), S(27,-37), S(-55,7),
        S(-27,-25), S(14,-50), S(-15,18), S(27,-70), S(9,-50), S(1,-25), S(20,-33), S(-40,-7),
        S(-32,-2), S(-8,-22), S(8,19), S(-56,8), S(21,-59), S(-15,-54), S(50,-72), S(-5,-7),
        S(-51,37), S(11,-11), S(11,54), S(-67,8), S(-18,-53), S(21,-43), S(16,-28), S(-21,-36),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(94,-533), S(-276,-3), S(-607,213), S(-255,81), S(-148,125), S(-498,103), S(375,168), S(-572,28),
        S(-219,191), S(16,17), S(159,-62), S(31,-105), S(84,-134), S(123,-98), S(5,-126), S(-287,99),
        S(-6,45), S(-38,-70), S(-65,29), S(118,-22), S(21,10), S(195,55), S(40,8), S(-37,-69),
        S(-35,-67), S(21,23), S(30,3), S(79,-36), S(116,-54), S(10,-53), S(62,17), S(201,-135),
        S(21,-57), S(130,-9), S(67,-8), S(1,30), S(51,41), S(61,87), S(170,-14), S(75,-98),
        S(68,-84), S(30,-52), S(42,52), S(-3,44), S(114,-17), S(49,-67), S(81,-34), S(50,29),
        S(182,-99), S(140,-85), S(97,-124), S(44,72), S(78,-21), S(-18,-109), S(14,-64), S(74,188),
        S(-294,-40), S(55,163), S(-116,-40), S(-52,219), S(37,8), S(208,-70), S(34,-132), S(-229,646),
    ],
    [
        S(82,-103), S(-90,115), S(-392,34), S(-319,-124), S(115,-51), S(-446,119), S(-68,142), S(-165,-54),
        S(-70,-5), S(-26,-46), S(-110,5), S(444,-200), S(-17,-21), S(-140,83), S(-64,22), S(-39,59),
        S(106,-141), S(153,-104), S(19,-4), S(-88,70), S(158,59), S(-186,179), S(53,18), S(-56,30),
        S(174,-95), S(84,-28), S(-53,86), S(94,35), S(-75,88), S(-67,100), S(0,61), S(21,-6),
        S(178,-137), S(28,-31), S(23,3), S(15,116), S(73,100), S(-26,155), S(8,36), S(150,-162),
        S(69,23), S(127,-33), S(23,-24), S(71,27), S(40,9), S(158,-50), S(79,111), S(61,12),
        S(199,87), S(26,42), S(-57,-50), S(66,-38), S(126,-118), S(-33,20), S(114,-4), S(92,-118),
        S(-335,100), S(102,-197), S(43,-24), S(-82,-5), S(-7,-89), S(43,-12), S(-333,-115), S(-74,47),
    ],
    [
        S(162,-10), S(227,-82), S(667,-206), S(677,-221), S(412,-157), S(70,-33), S(402,-62), S(247,-1),
        S(10,31), S(72,-29), S(208,-108), S(451,-142), S(343,-171), S(31,54), S(100,5), S(-30,86),
        S(-32,52), S(-115,49), S(-236,160), S(60,-56), S(45,20), S(62,-1), S(-43,-29), S(-131,69),
        S(-79,53), S(-98,28), S(159,-62), S(-57,-4), S(-49,-30), S(-215,72), S(-61,-5), S(14,-51),
        S(-170,49), S(-267,39), S(-128,19), S(-103,125), S(90,-116), S(-47,-18), S(-93,-46), S(-65,-3),
        S(-118,73), S(-100,6), S(-152,25), S(-173,62), S(-149,24), S(-97,33), S(-46,52), S(-174,35),
        S(-132,127), S(50,-137), S(45,-9), S(-143,106), S(-169,-0), S(-163,-30), S(-41,-58), S(-142,47),
        S(-148,92), S(-104,42), S(-79,56), S(-63,52), S(-84,41), S(-49,33), S(-150,33), S(-107,26),
    ],
    [
        S(335,-228), S(25,12), S(-325,300), S(-62,67), S(236,60), S(186,-97), S(363,-236), S(121,-256),
        S(-32,208), S(-23,33), S(2,240), S(87,111), S(193,-208), S(-10,171), S(5,-69), S(77,-181),
        S(49,8), S(-46,38), S(-98,75), S(26,116), S(32,177), S(-39,-7), S(-224,223), S(-56,84),
        S(-152,114), S(17,222), S(-46,101), S(-58,309), S(-55,69), S(-107,216), S(-90,-15), S(-11,-44),
        S(-90,170), S(-204,64), S(-102,116), S(-62,-41), S(-66,193), S(-127,198), S(-41,78), S(14,-150),
        S(66,-72), S(27,-75), S(-48,-205), S(-129,316), S(-115,29), S(-71,224), S(-98,218), S(70,106),
        S(-51,-267), S(-3,-29), S(25,-85), S(-44,6), S(-33,146), S(91,-249), S(38,-308), S(87,-187),
        S(-107,-130), S(33,-280), S(43,-283), S(-11,-76), S(54,-333), S(36,-173), S(113,-471), S(385,-62),
    ],
    [
        S(243,-241), S(697,-258), S(-449,-273), S(160,-62), S(-531,241), S(-872,287), S(-368,17), S(-344,-82),
        S(172,-76), S(607,10), S(-79,92), S(-494,73), S(-389,223), S(-398,153), S(-81,84), S(-644,65),
        S(192,7), S(-273,107), S(-173,158), S(-899,133), S(-299,122), S(-281,134), S(-207,135), S(-481,101),
        S(260,32), S(147,54), S(-239,82), S(591,-53), S(693,-74), S(383,-4), S(185,45), S(-211,85),
        S(19,-77), S(59,27), S(-84,75), S(15,37), S(320,-51), S(179,32), S(129,23), S(27,11),
        S(-4,-101), S(185,-34), S(195,-33), S(-24,39), S(-46,35), S(60,15), S(137,-73), S(183,-41),
        S(397,-133), S(26,12), S(117,-37), S(41,-32), S(108,-27), S(73,-6), S(174,-67), S(267,-148),
        S(-99,-27), S(189,-106), S(103,-84), S(101,-179), S(104,-34), S(80,-93), S(192,-65), S(162,-176),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(220,-828), S(-52,-18), S(-93,115), S(-56,93), S(-24,117), S(-25,137), S(-13,156), S(18,152), S(24,77), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-118,420), S(-105,150), S(-99,-76), S(-75,-43), S(-66,-1), S(-62,46), S(-60,50), S(-48,45), S(-43,49), S(19,-53), S(80,-18), S(200,-213), S(171,-94), S(206,-262), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-182,-215), S(-123,-158), S(-95,-34), S(-90,15), S(-78,25), S(-42,26), S(-47,71), S(-24,66), S(-6,62), S(-8,66), S(49,31), S(18,79), S(24,80), S(19,92), S(586,-205), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-329,-336), S(-329,-336), S(-473,-445), S(-131,-87), S(-10,141), S(-107,238), S(-104,-160), S(-113,-42), S(-101,52), S(-105,235), S(-101,204), S(-88,233), S(-98,290), S(-90,302), S(-73,288), S(-62,240), S(-29,170), S(-83,253), S(16,91), S(36,85), S(70,115), S(124,-41), S(113,38), S(587,-405), S(554,-374), S(486,-416), S(326,-238), S(118,-96)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(23,22), S(-19,60), S(24,78), S(-50,167), S(97,187), S(91,220), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(20,58), S(26,80), S(51,72), S(56,134), S(341,297), S(526,516), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(35,33), S(25,46), S(34,44), S(212,58), S(899,-145), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(40,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(34,33);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(214,-53);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(52,-3);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(30,17), S(22,22), S(23,35), S(1,130)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-61,74), S(-56,-15), S(-50,-2), S(-49,16), S(-16,11), S(23,-19), S(72,-23), S(158,-74), S(260,-100), S(226,-100), S(406,-205), S(77,-103), S(381,222), S(172,-135)];
#[rustfmt::skip]
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(2,66), S(23,-33), S(163,136), S(100,191), S(82,826), S(0,0)],
    [S(-0,43), S(-17,62), S(102,93), S(85,157), S(-86,653), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(-13,56), S(54,42), S(44,32), S(159,219), S(116,705), S(0,0)],
    [S(-14,16), S(44,46), S(-44,-32), S(104,273), S(6,821), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(-24,129), S(41,113), S(181,1), S(-12,-20), S(119,316), S(0,0)],
    [S(-27,54), S(29,24), S(90,-17), S(-90,-51), S(197,933), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(7,64), S(48,75), S(154,-173), S(81,-23), S(13,52), S(0,0)],
    [S(-7,19), S(15,93), S(-6,67), S(-67,96), S(-16,-55), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 60;

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
