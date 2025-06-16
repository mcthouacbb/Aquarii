use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait PolicyValueType:
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

impl PolicyValueType for f32 {}

pub trait PolicyValues {
    type Value: PolicyValueType;

    fn cap_bonus(pt: PieceType) -> Self::Value;
    fn pawn_protected_penalty(pt: PieceType) -> Self::Value;
    fn pawn_threat_evasion(pt: PieceType) -> Self::Value;
    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value;
    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value;
    fn passed_pawn_push(relative_rank: u8, phase: i32) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

const CAP_BONUS: [f32; 5] = [1.506, 2.612, 2.786, 2.674, 3.204];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [0.525, 2.216, 1.918, 3.175, 3.436];
const PAWN_THREAT_EVASION: [f32; 5] = [0.549, 2.414, 2.040, 2.175, 2.819];
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(51.436, 37.769), S(57.902, 24.507), S(72.803, 37.512), S(72.906, 37.045), S(87.933, 26.190), S(56.736, 24.792), S(51.256, 20.964), S(41.891, 48.755),
        S(42.674, 27.813), S(40.639, 26.116), S(66.202, 37.279), S(79.028, 31.776), S(77.014, 24.474), S(87.260, 18.591), S(57.223, 17.966), S(49.963, 30.239),
        S(-27.838, 1.429), S(23.488, -3.971), S(21.154, 13.790), S(43.632, 13.307), S(52.069, 3.422), S(37.276, -0.742), S(45.659, -12.839), S(-0.754, -1.884),
        S(-42.186, -31.493), S(-23.232, -17.624), S(-6.251, -7.930), S(12.632, 4.304), S(5.152, -0.624), S(0.724, -7.664), S(-1.462, -21.731), S(-20.540, -29.643),
        S(-23.069, -48.486), S(-30.294, -16.132), S(-3.652, -18.541), S(-21.418, 15.242), S(-5.966, 4.122), S(-13.365, -7.109), S(8.506, -20.954), S(5.627, -41.656),
        S(-30.246, -34.926), S(-25.683, -12.917), S(-27.532, -3.702), S(-61.353, 21.959), S(-48.108, 24.551), S(-4.248, 1.453), S(14.588, -17.263), S(-15.511, -27.820),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(7.685, -72.525), S(-0.201, -31.536), S(-55.950, -1.666), S(13.062, -15.242), S(2.902, -8.980), S(-26.950, -24.345), S(-16.131, -28.225), S(13.923, -64.564),
        S(-40.829, -14.468), S(-11.315, -4.010), S(-19.633, 13.330), S(-23.070, 13.621), S(-26.149, 11.344), S(-1.217, 3.213), S(-23.801, -3.751), S(-2.441, -30.174),
        S(-10.031, -10.942), S(5.496, 10.214), S(25.290, 16.161), S(5.143, 27.516), S(13.787, 19.384), S(29.643, 10.851), S(26.309, 1.841), S(2.886, -19.292),
        S(1.877, -0.816), S(10.486, 17.963), S(27.872, 25.460), S(40.212, 24.178), S(42.687, 23.924), S(33.377, 23.429), S(25.526, 14.465), S(40.293, -15.199),
        S(-3.299, -9.139), S(10.144, 8.496), S(26.297, 26.601), S(38.700, 21.278), S(44.130, 20.092), S(37.246, 19.970), S(27.932, 11.044), S(11.523, -18.331),
        S(-42.733, -12.524), S(3.573, 3.537), S(32.540, 3.714), S(31.157, 15.996), S(40.745, 16.889), S(34.755, 1.075), S(31.474, -7.379), S(-31.032, -8.561),
        S(-44.771, -24.738), S(-24.836, -15.012), S(-4.959, -1.489), S(-3.810, 5.497), S(3.036, 5.961), S(12.243, -1.671), S(-5.831, -18.120), S(-24.191, -22.387),
        S(-74.490, -27.124), S(-57.878, -40.721), S(-50.698, -12.148), S(-27.450, -15.758), S(-31.773, -18.532), S(-14.966, -21.949), S(-50.975, -38.327), S(-73.456, -17.374),
    ],
    [
        S(5.719, -20.150), S(-64.039, 0.706), S(10.050, -18.761), S(-5.596, -3.501), S(-24.429, -4.634), S(3.661, -15.169), S(-50.771, -4.458), S(12.462, -23.428),
        S(-34.253, -13.395), S(-14.010, -0.546), S(-23.318, 1.732), S(-57.835, 4.515), S(-50.302, 5.249), S(-28.408, -0.560), S(-19.494, 1.509), S(-20.512, -21.908),
        S(-26.150, -1.189), S(-0.182, 2.624), S(-7.401, 9.450), S(10.838, 1.229), S(2.386, 5.189), S(-2.024, 11.523), S(15.265, 0.088), S(-7.380, -2.365),
        S(-9.954, -4.240), S(3.449, 8.707), S(10.016, 12.235), S(8.225, 16.732), S(14.878, 15.541), S(11.521, 10.554), S(14.001, 6.971), S(-0.958, -10.080),
        S(3.052, -10.097), S(-4.012, 11.207), S(1.701, 17.253), S(27.296, 14.994), S(21.213, 13.581), S(10.218, 13.673), S(5.517, 4.513), S(16.726, -19.560),
        S(4.547, -8.123), S(17.523, 0.822), S(17.385, 12.945), S(9.051, 13.277), S(11.295, 18.130), S(12.111, 11.875), S(24.398, -3.447), S(8.335, -10.548),
        S(-9.842, -3.454), S(19.104, -11.091), S(13.611, -3.198), S(-9.111, 5.700), S(1.353, 4.827), S(26.307, -6.754), S(34.946, -12.469), S(1.912, -14.298),
        S(-15.614, -19.264), S(-17.902, -1.865), S(-14.939, -26.436), S(-29.965, -2.412), S(-15.249, -11.110), S(-17.894, -15.042), S(-7.793, -11.346), S(-21.464, -26.278),
    ],
    [
        S(11.424, 18.152), S(-10.191, 20.018), S(-37.339, 29.707), S(-37.102, 22.512), S(-28.340, 17.003), S(-28.617, 15.537), S(-5.783, 12.993), S(41.473, 5.535),
        S(-8.611, 16.572), S(-8.958, 19.063), S(-7.962, 20.439), S(-5.828, 11.310), S(-6.227, 5.805), S(-6.925, 5.531), S(-5.974, 6.150), S(27.605, 0.068),
        S(-12.097, 13.774), S(-8.825, 10.286), S(-7.086, 5.343), S(1.596, -3.825), S(8.160, -13.560), S(2.544, -9.608), S(19.405, -7.873), S(17.635, -7.048),
        S(-11.265, 10.421), S(-7.225, 3.657), S(3.556, 1.920), S(4.933, -4.522), S(14.107, -16.333), S(13.784, -15.496), S(18.944, -13.929), S(24.881, -13.001),
        S(-15.331, 6.221), S(-18.830, 4.318), S(-1.863, -1.621), S(12.208, -8.843), S(15.075, -13.576), S(0.829, -8.536), S(14.324, -14.558), S(8.502, -12.276),
        S(-27.870, 4.216), S(-8.826, -6.257), S(-2.887, -5.629), S(3.630, -9.587), S(11.380, -13.737), S(6.616, -16.543), S(33.098, -26.659), S(14.676, -21.293),
        S(-40.500, 3.060), S(-23.551, -2.365), S(-1.935, -7.811), S(1.227, -10.734), S(2.138, -14.847), S(3.304, -16.485), S(19.239, -25.089), S(-14.746, -11.765),
        S(3.194, -8.821), S(-9.844, 1.611), S(2.311, 3.343), S(16.770, -3.949), S(15.283, -10.782), S(10.974, -15.627), S(20.469, -16.888), S(10.232, -19.694),
    ],
    [
        S(-8.703, -1.347), S(-31.400, 14.019), S(-52.581, 34.853), S(-57.458, 32.651), S(-43.917, 26.917), S(-56.948, 27.468), S(-14.142, 3.252), S(5.888, -2.408),
        S(-10.907, -13.327), S(-21.684, 9.940), S(-33.060, 24.835), S(-73.421, 47.907), S(-84.271, 60.585), S(-43.874, 30.592), S(-27.127, 16.653), S(3.878, -2.522),
        S(-3.160, -18.822), S(4.222, -8.548), S(-15.607, 17.119), S(-8.101, 12.290), S(-11.983, 23.807), S(-7.399, 19.940), S(25.114, -5.282), S(12.269, -5.743),
        S(-14.110, -4.791), S(2.459, -1.310), S(-6.758, 10.181), S(-1.950, 17.007), S(4.707, 18.521), S(5.325, 19.063), S(17.602, 8.170), S(21.465, -8.123),
        S(7.574, -18.234), S(-6.355, 3.973), S(11.623, 0.350), S(19.519, 5.149), S(19.393, 8.839), S(19.010, 1.869), S(14.296, 3.631), S(22.112, -13.435),
        S(-3.842, -13.658), S(13.512, -12.760), S(23.275, -7.344), S(15.285, -0.696), S(23.449, -1.395), S(26.923, -5.588), S(39.416, -19.341), S(21.318, -21.196),
        S(-11.005, -16.228), S(7.508, -16.292), S(22.295, -25.746), S(17.254, -13.762), S(20.345, -15.260), S(31.623, -30.565), S(34.525, -36.814), S(19.666, -42.027),
        S(-0.688, -21.343), S(-29.674, -4.096), S(-9.639, -11.144), S(22.324, -64.231), S(3.582, -22.995), S(-12.223, -7.481), S(0.379, -20.322), S(-13.750, -23.577),
    ],
    [
        S(28.188, -37.386), S(35.269, -19.842), S(27.232, -13.630), S(-22.316, -0.320), S(-11.555, 1.151), S(14.696, 3.631), S(31.783, 0.871), S(57.632, -33.570),
        S(-6.307, -12.125), S(-1.505, 20.165), S(-23.518, 19.781), S(-9.978, 18.648), S(-41.746, 32.006), S(-5.416, 35.300), S(23.404, 26.424), S(-4.995, 8.604),
        S(-58.173, 3.435), S(-4.415, 23.306), S(-46.177, 35.667), S(-66.582, 47.781), S(-48.320, 50.553), S(4.001, 44.317), S(8.146, 36.508), S(-15.225, 13.751),
        S(-62.232, 2.716), S(-48.837, 26.070), S(-71.923, 41.805), S(-119.882, 56.673), S(-102.658, 56.399), S(-85.731, 51.772), S(-67.653, 37.897), S(-88.263, 16.549),
        S(-63.545, -7.754), S(-61.566, 17.366), S(-73.364, 32.975), S(-107.282, 50.654), S(-108.968, 50.699), S(-78.625, 38.389), S(-72.433, 24.050), S(-97.018, 8.447),
        S(-22.821, -20.466), S(-2.114, -1.272), S(-41.808, 17.466), S(-58.494, 29.858), S(-57.280, 30.462), S(-48.385, 20.789), S(-19.344, 4.433), S(-26.435, -13.967),
        S(33.753, -38.812), S(15.985, -17.277), S(5.953, -4.147), S(-23.567, 4.084), S(-23.880, 6.980), S(0.187, -3.768), S(25.923, -18.281), S(19.454, -35.194),
        S(-6.641, -66.531), S(63.104, -57.560), S(45.839, -42.911), S(-11.317, -36.020), S(49.036, -52.763), S(4.954, -40.845), S(48.973, -51.817), S(11.957, -71.087),
    ],
];
const THREAT: [[f32; 5]; 4] = [
    [0.162, -0.039, 0.714, 0.672, 0.782],
    [0.057, 0.649, -0.235, 0.743, 0.564],
    [0.232, 0.613, 0.608, -0.042, 1.086],
    [0.119, 0.378, 0.076, 0.050, 0.135],
];
const PASSED_PAWN_PUSH: [(f32, f32); 8] = [
    S(0.000, 0.000),
    S(-0.136, 0.061),
    S(-0.098, 0.676),
    S(0.453, 1.038),
    S(0.557, 1.268),
    S(1.948, 1.685),
    S(0.000, 0.000),
    S(0.000, 0.000),
];
const PROMO_BONUS: [f32; 2] = [1.105, -1.818];
const BAD_SEE_PENALTY: f32 = -2.584;
const CHECK_BONUS: f32 = 0.506;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        CAP_BONUS[pt as usize]
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        PAWN_PROTECTED_PENALTY[pt as usize]
    }

    fn pawn_threat_evasion(pt: PieceType) -> Self::Value {
        PAWN_THREAT_EVASION[pt as usize]
    }

    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value {
        (PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].0 * phase as f32
            + PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].1 * (24 - phase) as f32)
            / 24.0
    }

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[moving as usize - PieceType::Knight as usize][threatened as usize]
    }

    fn passed_pawn_push(relative_rank: u8, phase: i32) -> Self::Value {
        (PASSED_PAWN_PUSH[relative_rank as usize].0 * phase as f32
            + PASSED_PAWN_PUSH[relative_rank as usize].1 * (24 - phase) as f32)
            / 24.0
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

pub fn get_policy(board: &Board, mv: Move) -> f32 {
    get_policy_impl::<PolicyParams>(board, mv)
}

pub fn get_policy_impl<Params: PolicyValues>(board: &Board, mv: Move) -> Params::Value {
    let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let pawn_protected = attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
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

    let pawn_threat_evasion = if pawn_protected.has(mv.from_sq())
        && !pawn_protected.has(mv.to_sq())
        && moving_piece.piece_type() != PieceType::King
    {
        Params::pawn_threat_evasion(moving_piece.piece_type())
    } else {
        Params::Value::default()
    };

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

    let threat_score = if mv.kind() == MoveKind::None
        && moving_piece.piece_type() != PieceType::King
        && moving_piece.piece_type() != PieceType::Pawn
    {
        let occ_after =
            board.occ() | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
        let attacks_after =
            attacks::piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after);

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

    let bad_see = !see::see(board, mv, 0);
    let bad_see_penalty = if bad_see && !pawn_protected.has(mv.to_sq()) {
        Params::bad_see_penalty()
    } else {
        Params::Value::default()
    };

    let their_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let mut pawn_policy = Params::Value::default();
    if mv.kind() == MoveKind::None && moving_piece.piece_type() == PieceType::Pawn {
        let relative_rank = mv.from_sq().relative_sq(board.stm()).rank();
        let stoppers = their_pawns & attacks::passed_pawn_span(board.stm(), mv.from_sq());
        if stoppers.empty() && (mv.from_sq() - mv.to_sq()).abs() == 8 {
            pawn_policy += Params::passed_pawn_push(relative_rank, phase);
        }
    }

    let check_bonus = if board.gives_direct_check(mv) {
        Params::check_bonus()
    } else {
        Params::Value::default()
    };

    cap_bonus + promo_bonus + pawn_threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
        + pawn_policy
}
