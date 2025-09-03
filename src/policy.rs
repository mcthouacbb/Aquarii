use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait PolicyScoreType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
{
}

impl PolicyScoreType for f32 {}

pub trait PolicyValues {
    type Value: PolicyScoreType;

    fn cap_bonus(pt: PieceType) -> Self::Value;
    fn pawn_protected_penalty(pt: PieceType) -> Self::Value;
    fn threat_evasion(threat: PieceType, moving: PieceType) -> Self::Value;
    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value;
    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

#[rustfmt::skip]
const CAP_BONUS: [f32; 5] = [1.409, 2.483, 2.663, 2.669, 3.602];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.514, 2.110, 1.796, 2.744, 3.189];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 5] = [
    [0.497, 2.720, 2.462, 2.171, 2.935],
    [0.405, 0.313, 1.525, 1.984, 2.856],
    [0.242, 0.486, 0.262, 1.877, 2.642],
    [0.369, 0.490, 0.649, 0.467, 2.620],
    [0.084, 0.372, 0.523, 0.584, 0.917],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(141.650, 102.638), S(49.958, 109.686), S(117.601, 106.565), S(100.359, 110.032), S(102.979, 98.984), S(106.392, 67.526), S(74.693, 66.571), S(127.441, 92.042),
        S(55.142, 37.607), S(47.923, 33.978), S(58.282, 41.611), S(87.690, 40.155), S(81.236, 27.270), S(75.850, 13.695), S(70.375, 16.595), S(66.607, 33.535),
        S(-16.363, -12.821), S(35.908, -19.733), S(20.067, -7.819), S(48.985, -11.244), S(54.127, -12.682), S(33.553, -19.599), S(44.060, -21.172), S(-1.740, -15.942),
        S(-40.942, -51.306), S(-21.271, -36.068), S(4.904, -36.943), S(29.211, -31.920), S(11.414, -26.013), S(19.398, -40.498), S(-12.072, -33.333), S(-30.944, -40.782),
        S(-30.446, -69.750), S(-27.282, -39.528), S(0.537, -41.106), S(-22.646, -14.321), S(-4.683, -19.258), S(-16.490, -29.507), S(6.589, -40.227), S(-11.129, -54.443),
        S(-38.746, -50.621), S(-11.136, -38.315), S(-38.634, -10.845), S(-63.377, 13.582), S(-62.942, 16.076), S(6.113, -21.852), S(16.140, -32.020), S(-24.943, -42.011),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(-10.789, -20.164), S(-3.068, -17.967), S(-45.778, -1.412), S(6.565, -11.524), S(12.084, -14.863), S(-24.542, -1.160), S(17.701, -23.927), S(29.897, -57.621),
        S(-57.637, 0.260), S(-8.280, 3.688), S(-9.069, 9.706), S(-18.422, 13.985), S(-26.361, 22.508), S(-0.361, 3.977), S(-11.235, -6.015), S(-33.085, 2.599),
        S(6.244, -8.462), S(4.039, 11.343), S(26.143, 6.895), S(25.802, 16.916), S(18.419, 12.892), S(36.690, 14.196), S(34.658, 1.820), S(27.246, -13.637),
        S(4.035, -0.896), S(10.722, 19.282), S(33.684, 15.321), S(39.800, 20.559), S(51.617, 11.996), S(47.686, 12.907), S(23.397, 16.246), S(38.273, -0.470),
        S(-5.795, -10.475), S(18.182, 1.720), S(25.499, 22.501), S(38.988, 20.610), S(42.930, 21.270), S(41.789, 19.358), S(33.667, 9.264), S(12.642, -2.699),
        S(-38.382, -0.955), S(16.343, -6.519), S(22.078, 10.008), S(39.111, 9.053), S(36.175, 20.392), S(36.679, 2.869), S(43.463, -6.631), S(-26.295, 0.557),
        S(-94.537, -13.727), S(-15.418, -8.902), S(-0.103, 4.665), S(-11.717, 14.195), S(10.306, -4.275), S(18.420, -2.014), S(2.093, 0.371), S(2.295, -40.172),
        S(-99.930, -12.883), S(-52.790, -23.346), S(-52.485, -7.570), S(-34.651, -8.528), S(-15.321, -21.513), S(-2.381, -6.004), S(-52.100, -36.042), S(3.126, -83.818),
    ],
    [
        S(3.222, -30.641), S(-80.989, 16.692), S(-13.720, -3.158), S(-20.767, -4.256), S(-34.816, 1.513), S(-6.458, -8.393), S(-84.316, 24.027), S(8.543, 6.605),
        S(-28.982, -15.115), S(-18.021, -0.499), S(-47.665, 9.506), S(-70.209, 10.818), S(-69.613, 8.721), S(-23.366, 5.934), S(-26.745, 5.703), S(-34.891, -4.896),
        S(-32.841, -10.529), S(13.198, -4.445), S(-22.277, 12.709), S(8.200, -5.288), S(11.145, 3.176), S(-16.157, 19.073), S(28.314, -8.372), S(-2.387, -4.609),
        S(-2.618, -10.896), S(-6.149, 11.403), S(18.132, 4.711), S(20.810, 5.687), S(21.193, 6.983), S(8.415, 8.828), S(10.052, 6.487), S(23.400, -17.839),
        S(16.633, -10.685), S(4.984, -3.262), S(10.554, 10.402), S(38.761, 10.258), S(25.523, 9.135), S(17.143, 8.366), S(12.701, 11.246), S(15.373, -10.894),
        S(13.109, -8.478), S(51.181, -7.491), S(23.894, 4.304), S(9.931, 11.235), S(12.217, 17.714), S(33.769, 3.169), S(39.343, -14.235), S(-2.567, -2.572),
        S(12.498, -2.642), S(24.168, -8.591), S(26.191, -10.960), S(-4.816, -1.050), S(14.908, -11.207), S(34.748, -12.970), S(43.281, -28.667), S(17.366, -24.365),
        S(-29.595, 4.065), S(-13.435, -7.589), S(-8.995, -17.033), S(-47.784, 9.825), S(-15.697, -13.885), S(-33.134, -0.984), S(-26.010, -6.006), S(-39.281, 0.994),
    ],
    [
        S(17.488, 16.505), S(24.786, 3.284), S(-1.441, 2.094), S(-2.601, -4.091), S(-17.461, 3.084), S(-33.374, 8.765), S(7.270, 10.602), S(37.340, 7.300),
        S(4.369, 26.328), S(7.807, 12.086), S(26.836, 2.225), S(12.634, 1.889), S(15.104, -1.222), S(4.471, -0.196), S(0.175, 7.451), S(11.412, 16.327),
        S(-16.341, 23.869), S(-6.495, 9.814), S(-1.226, 2.385), S(15.432, -6.904), S(6.737, -8.468), S(-0.596, -5.161), S(8.550, -0.701), S(6.828, 10.406),
        S(-6.123, 18.669), S(-11.841, 10.843), S(8.263, 3.247), S(16.122, -7.604), S(19.736, -11.674), S(13.486, -4.451), S(9.921, 5.877), S(21.674, -2.323),
        S(-14.093, 13.199), S(-18.343, 7.865), S(17.536, -2.349), S(5.238, -0.652), S(23.572, -12.809), S(4.278, -6.583), S(14.862, -6.998), S(-5.910, 3.927),
        S(-30.252, 9.887), S(9.033, -11.948), S(9.414, -11.299), S(-7.473, -2.513), S(20.606, -19.291), S(8.643, -15.073), S(20.169, -11.247), S(2.220, -9.436),
        S(-57.442, 10.341), S(-22.496, 0.901), S(1.719, -8.693), S(5.561, -18.292), S(6.404, -20.867), S(0.459, -3.717), S(14.832, -19.801), S(-47.755, -5.002),
        S(-8.685, 0.483), S(-13.334, 0.666), S(14.777, -8.363), S(18.766, -7.963), S(38.107, -26.745), S(20.784, -19.547), S(-1.307, -2.688), S(11.143, -17.566),
    ],
    [
        S(-3.806, 14.528), S(-11.503, 16.593), S(-43.311, 35.345), S(-56.226, 38.066), S(-41.480, 31.790), S(-57.726, 33.311), S(-18.200, 20.967), S(15.297, 4.439),
        S(-19.286, 7.818), S(-11.504, 13.454), S(-12.644, 15.181), S(-49.614, 31.948), S(-49.986, 39.832), S(-23.819, 21.552), S(-18.445, 9.881), S(-1.901, 6.316),
        S(-15.094, 10.704), S(4.554, -2.318), S(-6.201, 11.728), S(2.270, 8.686), S(-0.944, 12.920), S(6.612, 10.890), S(33.716, -8.738), S(25.185, -12.556),
        S(-5.458, 3.918), S(3.807, 8.883), S(0.575, 13.661), S(10.767, 12.805), S(15.532, 12.585), S(8.481, 10.564), S(23.437, -1.168), S(28.994, -16.963),
        S(-7.016, -4.492), S(-5.844, 4.439), S(14.413, 5.179), S(22.354, 4.943), S(14.968, 10.870), S(15.166, 9.690), S(14.947, 8.713), S(22.347, -13.991),
        S(-9.983, -1.565), S(11.297, -5.407), S(22.713, -5.073), S(9.499, 9.758), S(13.824, 6.390), S(15.646, 10.436), S(33.721, -13.344), S(14.375, -24.552),
        S(-35.479, 23.095), S(-4.625, -4.907), S(19.899, -43.095), S(3.611, 1.684), S(1.500, 9.682), S(25.893, -24.841), S(23.811, -11.644), S(-21.531, -2.973),
        S(-22.614, 6.921), S(-32.814, -12.174), S(-4.488, -32.442), S(12.580, -73.189), S(-4.220, -32.258), S(-20.876, -12.957), S(-53.021, 16.362), S(-57.376, 13.387),
    ],
    [
        S(56.985, 5.078), S(30.648, 10.620), S(17.902, 23.900), S(-112.959, 54.770), S(-158.703, 82.485), S(-152.866, 81.688), S(-67.995, 43.932), S(24.840, 14.999),
        S(-32.511, 19.067), S(0.730, 29.400), S(-10.566, 33.660), S(-42.504, 51.987), S(-74.644, 69.143), S(-105.630, 67.811), S(-39.661, 50.402), S(-18.146, 26.660),
        S(-34.881, 23.344), S(-19.694, 37.786), S(-36.244, 39.912), S(-42.000, 41.080), S(-41.532, 44.204), S(-57.798, 55.329), S(-61.981, 64.420), S(-81.649, 55.062),
        S(-55.466, 18.877), S(-15.997, 32.651), S(-8.089, 31.448), S(-28.666, 24.799), S(-10.495, 28.456), S(-34.105, 40.743), S(-71.526, 51.455), S(-78.711, 33.343),
        S(-20.966, -4.338), S(16.203, 8.878), S(-23.573, 18.685), S(-21.796, 21.450), S(-20.054, 20.236), S(-28.399, 27.729), S(-44.541, 23.620), S(-70.109, 12.648),
        S(42.791, -27.656), S(35.130, -12.335), S(-24.787, 6.356), S(-33.793, 12.565), S(-51.393, 20.128), S(-21.271, 12.355), S(3.262, -4.400), S(-15.936, -16.441),
        S(78.300, -52.407), S(38.661, -30.603), S(21.989, -19.001), S(-40.906, -1.586), S(-38.600, 1.634), S(-9.474, -8.522), S(44.183, -28.103), S(25.382, -41.860),
        S(-10.381, -68.862), S(58.818, -56.237), S(23.201, -43.726), S(-33.189, -36.069), S(39.241, -60.622), S(-8.191, -49.011), S(51.665, -58.112), S(14.003, -75.953),
    ],
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.951, 0.683, 0.800, 0.581, 0.751],
    [0.064, -0.009, 0.811, 0.633, 0.768],
    [0.082, 0.506, -0.074, 0.687, 0.719],
    [0.280, 0.555, 0.575, 0.085, 1.040],
    [0.091, 0.200, 0.069, 0.153, -0.172],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.029, -1.039];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.697;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.571;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        CAP_BONUS[pt as usize]
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        PAWN_PROTECTED_PENALTY[pt as usize]
    }

    fn threat_evasion(threat: PieceType, moving: PieceType) -> Self::Value {
        THREAT_EVASION[threat as usize][moving as usize]
    }

    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value {
        (PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].0 * phase as f32
            + PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].1 * (24 - phase) as f32)
            / 24.0
    }

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[moving as usize - PieceType::Pawn as usize][threatened as usize]
    }

    fn promo_bonus(pt: PieceType) -> Self::Value {
        match pt {
            PieceType::Queen => PROMO_BONUS[0],
            _ => PROMO_BONUS[1],
        }
    }

    fn bad_see_penalty() -> Self::Value {
        BAD_SEE_PENALTY
    }

    fn check_bonus() -> Self::Value {
        CHECK_BONUS
    }
}

#[derive(Debug, Clone)]
pub struct PolicyData {
    attacked: Bitboard,
    attacked_by: [Bitboard; 6],
}

impl PolicyData {
    pub fn new(board: &Board) -> Self {
        let mut result: PolicyData = Self {
            attacked: Bitboard::NONE,
            attacked_by: [Bitboard::NONE; 6],
        };

        let stm = board.stm();

        result.add_attacks(
            PieceType::Pawn,
            attacks::pawn_attacks_bb(
                !stm,
                board.colored_pieces(Piece::new(!stm, PieceType::Pawn)),
            ),
        );

        result.add_attacks(PieceType::King, attacks::king_attacks(board.king_sq(!stm)));

        for pt in [
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            let mut bb = board.colored_pieces(Piece::new(!stm, pt));
            while bb.any() {
                let sq = bb.poplsb();
                let attacks = attacks::piece_attacks(pt, sq, board.occ());
                result.add_attacks(pt, attacks);
            }
        }

        result
    }

    fn add_attacks(&mut self, pt: PieceType, attacks: Bitboard) {
        self.attacked |= attacks;
        self.attacked_by[pt as usize] |= attacks;
    }

    fn attacked(&self) -> Bitboard {
        self.attacked
    }

    fn attacked_by(&self, pt: PieceType) -> Bitboard {
        self.attacked_by[pt as usize]
    }
}

pub fn get_policy(board: &Board, mv: Move, data: &PolicyData) -> f32 {
    get_policy_impl::<PolicyParams>(board, mv, data)
}

pub fn get_policy_impl<Params: PolicyValues>(
    board: &Board,
    mv: Move,
    data: &PolicyData,
) -> Params::Value {
    let pawn_protected = data.attacked_by(PieceType::Pawn);
    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let captured_piece = board.piece_at(mv.to_sq());
    let cap_bonus = if let Some(captured) = captured_piece {
        Params::cap_bonus(captured.piece_type())
    } else {
        Params::Value::default()
    };

    let pawn_protected_penalty = if pawn_protected.has(mv.to_sq()) {
        Params::pawn_protected_penalty(moving_piece.piece_type())
    } else {
        Params::Value::default()
    };

    let mut threat_evasion = Params::Value::default();
    if moving_piece.piece_type() != PieceType::King && data.attacked().has(mv.from_sq()) {
        for pt in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            let from_threat = data.attacked_by(pt).has(mv.from_sq());
            let to_threat = data.attacked_by(pt).has(mv.to_sq());
            if from_threat && !to_threat {
                threat_evasion += Params::threat_evasion(pt, moving_piece.piece_type());
            }

            if from_threat || to_threat {
                break;
            }
        }
    }

    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount())
    .min(24) as i32;
    let psqt = if mv.kind() != MoveKind::Promotion {
        Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.to_sq(), phase)
            - Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.from_sq(), phase)
    } else {
        Params::Value::default()
    };

    let threat_score =
        if mv.kind() == MoveKind::None && moving_piece.piece_type() != PieceType::King {
            let occ_after = board.occ()
                | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
            let attacks_after = if moving_piece.piece_type() != PieceType::Pawn {
                attacks::piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after)
            } else {
                attacks::pawn_attacks(board.stm(), mv.to_sq())
            };

            let mut threats =
                attacks_after & board.colors(!board.stm()) & !board.pieces(PieceType::King);
            let mut score = Params::Value::default();
            while threats.any() {
                let threat = threats.poplsb();
                score += Params::threat(
                    moving_piece.piece_type(),
                    board.piece_at(threat).unwrap().piece_type(),
                );
            }

            score
        } else {
            Params::Value::default()
        };

    let promo_bonus = if mv.kind() == MoveKind::Promotion {
        Params::promo_bonus(mv.promo_piece())
    } else {
        Params::Value::default()
    };

    let bad_see_penalty = if !see::see(board, mv, 0) && !pawn_protected.has(mv.to_sq()) {
        Params::bad_see_penalty()
    } else {
        Params::Value::default()
    };

    let check_bonus = if board.gives_direct_check(mv) {
        Params::check_bonus()
    } else {
        Params::Value::default()
    };

    cap_bonus + promo_bonus + threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
