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
    fn king_ring_attacks(moving: PieceType, attacks: u32, phase: i32) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

const CAP_BONUS: [f32; 5] = [1.486, 2.530, 2.693, 2.677, 3.225];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [0.666, 2.337, 1.920, 3.120, 3.383];
const PAWN_THREAT_EVASION: [f32; 5] = [0.329, 2.300, 2.094, 2.217, 2.690];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(131.043, 144.003), S(96.252, 131.245), S(118.983, 146.050), S(112.549, 148.021), S(121.021, 136.796), S(71.299, 132.077), S(67.486, 129.528), S(104.924, 157.989),
        S(47.389, 47.372), S(45.873, 38.085), S(73.481, 52.766), S(86.759, 47.419), S(84.325, 37.638), S(91.431, 30.814), S(58.563, 31.334), S(52.650, 49.931),
        S(-30.170, -30.154), S(25.235, -38.832), S(23.366, -22.093), S(46.253, -19.817), S(54.224, -30.176), S(38.440, -36.123), S(44.211, -44.458), S(-5.911, -33.010),
        S(-45.308, -92.009), S(-26.252, -77.225), S(-8.852, -70.512), S(9.911, -56.531), S(1.587, -60.079), S(-3.630, -67.710), S(-6.772, -78.222), S(-26.644, -87.777),
        S(-26.785, -116.160), S(-35.817, -83.468), S(-8.874, -89.233), S(-26.876, -57.527), S(-12.788, -66.271), S(-19.856, -73.600), S(1.085, -85.000), S(-0.789, -107.314),
        S(-35.356, -100.285), S(-32.598, -79.537), S(-34.971, -73.293), S(-68.003, -48.900), S(-56.285, -44.574), S(-12.758, -64.730), S(5.227, -80.527), S(-23.618, -91.242),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(1.688, -67.464), S(-13.579, -27.765), S(-67.021, -0.128), S(-11.810, -10.828), S(-18.827, -6.779), S(-50.954, -20.776), S(-35.011, -25.102), S(-4.081, -59.297),
        S(-40.674, -13.512), S(-19.919, -1.308), S(-24.303, 15.077), S(-40.869, 15.641), S(-35.839, 11.413), S(-16.022, 5.126), S(-32.923, -2.312), S(-17.454, -25.993),
        S(-11.816, -8.699), S(1.037, 11.380), S(17.959, 17.795), S(-4.943, 28.349), S(1.571, 20.750), S(15.761, 12.600), S(16.532, 2.661), S(-10.612, -17.095),
        S(2.616, -0.231), S(8.206, 19.060), S(21.921, 26.743), S(32.948, 24.790), S(35.621, 23.619), S(23.055, 24.645), S(19.259, 15.625), S(29.525, -12.613),
        S(2.345, -10.752), S(12.970, 7.764), S(22.141, 26.818), S(34.731, 21.318), S(35.693, 20.385), S(31.883, 19.508), S(17.968, 11.819), S(13.961, -22.000),
        S(-36.768, -13.854), S(8.229, 2.336), S(38.058, 1.833), S(33.834, 14.579), S(43.077, 14.934), S(40.212, -1.549), S(34.868, -9.492), S(-24.847, -11.124),
        S(-38.349, -24.635), S(-20.239, -16.064), S(-0.042, -3.338), S(1.491, 3.866), S(8.315, 4.013), S(17.030, -3.528), S(-2.133, -20.028), S(-18.047, -23.525),
        S(-71.103, -26.793), S(-54.452, -41.689), S(-45.128, -12.998), S(-21.921, -16.985), S(-25.584, -20.674), S(-9.772, -23.369), S(-46.837, -39.526), S(-69.741, -16.973),
    ],
    [
        S(5.698, -20.257), S(-66.478, 0.751), S(8.628, -18.914), S(-11.856, -2.608), S(-29.517, -4.235), S(0.093, -15.508), S(-52.322, -4.645), S(10.721, -23.544),
        S(-33.052, -13.440), S(-12.007, -0.836), S(-24.663, 2.246), S(-57.543, 4.134), S(-49.593, 4.265), S(-29.965, -0.804), S(-17.468, 0.766), S(-19.210, -23.192),
        S(-24.559, -0.866), S(-0.703, 3.141), S(-7.828, 9.671), S(8.090, 1.167), S(0.926, 4.939), S(-5.158, 12.112), S(14.012, 0.500), S(-8.545, -2.126),
        S(-10.664, -3.573), S(2.632, 9.233), S(7.008, 12.695), S(6.774, 16.806), S(11.756, 15.658), S(9.397, 10.917), S(10.771, 7.322), S(-2.012, -9.927),
        S(3.584, -9.860), S(-6.233, 11.542), S(0.596, 17.368), S(25.330, 15.011), S(21.673, 12.461), S(7.899, 13.359), S(4.647, 4.358), S(15.312, -19.144),
        S(3.744, -7.567), S(17.738, 0.681), S(17.393, 13.005), S(9.724, 12.549), S(11.999, 17.345), S(13.492, 11.257), S(24.094, -3.600), S(7.928, -10.284),
        S(-8.825, -3.450), S(19.152, -11.132), S(14.833, -3.654), S(-8.444, 5.205), S(2.020, 4.512), S(27.739, -7.123), S(35.375, -12.898), S(1.605, -13.978),
        S(-15.221, -19.007), S(-17.034, -1.951), S(-14.897, -26.739), S(-28.626, -2.841), S(-14.089, -11.641), S(-18.304, -14.990), S(-6.719, -11.497), S(-19.785, -26.759),
    ],
    [
        S(11.575, 17.793), S(-10.640, 19.606), S(-38.580, 29.345), S(-38.839, 22.476), S(-30.264, 16.731), S(-30.326, 15.465), S(-8.013, 13.269), S(39.931, 5.387),
        S(-8.058, 16.211), S(-8.751, 18.619), S(-9.701, 20.858), S(-8.825, 12.303), S(-9.680, 6.779), S(-9.652, 6.430), S(-7.971, 7.199), S(26.723, -0.035),
        S(-11.522, 13.000), S(-7.895, 10.099), S(-8.398, 6.333), S(-1.081, -2.097), S(4.634, -11.314), S(-0.674, -7.377), S(17.328, -6.297), S(15.310, -6.256),
        S(-9.369, 9.875), S(-5.590, 3.612), S(3.652, 2.891), S(4.105, -2.755), S(12.208, -13.708), S(10.536, -12.935), S(18.638, -13.094), S(24.890, -13.112),
        S(-13.280, 5.649), S(-16.598, 3.738), S(-0.105, -1.215), S(12.606, -7.717), S(13.860, -11.554), S(-0.844, -6.554), S(14.244, -13.829), S(9.486, -12.773),
        S(-25.642, 3.862), S(-6.424, -6.567), S(-0.867, -6.061), S(4.605, -9.284), S(11.674, -13.077), S(6.610, -15.959), S(34.616, -26.833), S(16.277, -21.897),
        S(-38.595, 2.956), S(-21.979, -2.403), S(-0.135, -7.593), S(2.580, -10.191), S(3.211, -14.216), S(3.930, -16.458), S(20.701, -25.004), S(-13.106, -12.001),
        S(3.991, -9.603), S(-7.896, 1.136), S(3.983, 2.832), S(18.211, -4.299), S(16.790, -11.116), S(12.147, -16.143), S(22.250, -17.425), S(10.931, -20.173),
    ],
    [
        S(-10.280, -1.775), S(-32.267, 13.256), S(-54.634, 34.473), S(-61.627, 33.765), S(-47.819, 27.644), S(-58.999, 27.435), S(-15.698, 2.882), S(4.223, -3.101),
        S(-10.276, -15.148), S(-19.260, 7.958), S(-29.927, 22.704), S(-69.520, 46.288), S(-61.085, 52.672), S(-39.608, 29.465), S(-18.300, 13.397), S(3.176, -3.786),
        S(-3.889, -20.188), S(2.362, -7.689), S(-16.668, 18.420), S(-8.695, 13.760), S(-10.708, 24.606), S(-7.019, 21.292), S(25.061, -3.595), S(9.872, -5.334),
        S(-14.959, -4.607), S(1.344, -1.141), S(-10.254, 11.760), S(-4.820, 19.330), S(-0.716, 22.404), S(0.872, 22.298), S(12.336, 11.266), S(18.163, -7.144),
        S(6.600, -19.190), S(-9.493, 3.856), S(9.152, 0.701), S(15.816, 6.314), S(15.651, 10.753), S(14.290, 3.852), S(10.828, 5.001), S(18.279, -13.167),
        S(-5.687, -14.657), S(12.198, -13.708), S(21.764, -7.889), S(13.757, -1.664), S(21.349, -1.209), S(24.310, -5.239), S(36.879, -19.563), S(18.131, -21.570),
        S(-11.509, -18.252), S(6.840, -17.841), S(21.617, -26.965), S(16.397, -15.246), S(19.243, -15.624), S(30.682, -31.047), S(33.877, -38.144), S(18.514, -43.882),
        S(-0.646, -24.220), S(-30.166, -5.089), S(-10.234, -12.695), S(21.040, -64.569), S(3.071, -23.872), S(-13.271, -8.167), S(0.421, -22.231), S(-13.939, -25.884),
    ],
    [
        S(23.094, -34.049), S(40.709, -18.781), S(43.331, -14.781), S(-4.086, -2.193), S(7.165, -1.161), S(31.215, 1.975), S(45.468, -0.464), S(52.913, -30.552),
        S(6.297, -12.442), S(15.093, 18.105), S(-4.611, 17.573), S(12.432, 16.071), S(-20.810, 29.303), S(13.983, 33.001), S(41.104, 24.161), S(9.550, 7.030),
        S(-38.977, 0.942), S(14.134, 20.960), S(-24.573, 32.890), S(-45.688, 45.081), S(-28.349, 47.701), S(22.679, 41.773), S(26.288, 34.101), S(1.702, 11.446),
        S(-47.549, 1.151), S(-35.663, 24.496), S(-56.361, 39.800), S(-100.680, 53.812), S(-83.858, 53.263), S(-69.960, 49.774), S(-53.923, 36.229), S(-72.173, 14.653),
        S(-54.150, -8.518), S(-52.682, 16.469), S(-65.052, 32.161), S(-98.128, 49.159), S(-99.921, 49.244), S(-71.307, 37.689), S(-67.097, 23.723), S(-90.967, 8.097),
        S(-22.344, -19.096), S(-2.347, 0.105), S(-41.475, 18.178), S(-59.045, 30.656), S(-58.041, 30.977), S(-49.356, 21.756), S(-21.347, 5.748), S(-28.434, -12.117),
        S(31.049, -36.528), S(13.131, -15.157), S(2.320, -2.318), S(-28.026, 6.118), S(-28.343, 8.848), S(-4.524, -1.455), S(22.331, -16.192), S(15.676, -32.801),
        S(-10.687, -62.935), S(59.264, -55.361), S(40.993, -40.459), S(-16.168, -33.818), S(43.193, -50.642), S(-0.024, -38.480), S(43.840, -49.546), S(7.702, -68.920),
    ],
];
const THREAT: [[f32; 5]; 4] = [
    [0.068, -0.207, 0.664, 0.568, 0.618],
    [0.042, 0.620, -0.206, 0.653, 0.523],
    [0.268, 0.573, 0.557, -0.001, 1.080],
    [0.121, 0.406, 0.111, 0.053, 0.100],
];
const KING_RING_ATTACKS: [[(f32, f32); 9]; 4] = [
    [S(0.000, 0.000), S(0.467, 0.141), S(0.459, 0.118), S(0.494, -0.073), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),],
    [S(0.000, 0.000), S(0.153, 0.123), S(0.125, 0.182), S(0.221, 0.054), S(-0.219, -0.001), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),],
    [S(0.000, 0.000), S(0.194, 0.169), S(0.242, -0.121), S(0.225, -0.124), S(0.117, -0.118), S(0.204, -0.184), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),],
    [S(0.000, 0.000), S(0.184, 0.183), S(0.153, 0.127), S(0.239, 0.009), S(0.209, -0.075), S(-0.082, -0.100), S(0.028, -0.130), S(-1.267, 0.458), S(0.000, 0.000),],
];
const PROMO_BONUS: [f32; 2] = [1.144, -1.865];
const BAD_SEE_PENALTY: f32 = -2.561;
const CHECK_BONUS: f32 = 0.460;

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
        (PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].0 * phase.min(24) as f32
            + PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].1
                * (24 - phase.min(24)) as f32)
            / 24.0
    }

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[moving as usize - PieceType::Knight as usize][threatened as usize]
    }

    fn king_ring_attacks(moving: PieceType, attacks: u32, phase: i32) -> Self::Value {
        let (mg, eg) = KING_RING_ATTACKS[moving as usize - PieceType::Knight as usize][attacks as usize];
        (mg * phase.min(24) as f32 + eg * (24 - phase.min(24)) as f32) / 24.0
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
        + board.pieces(PieceType::Knight).popcount()) as i32;
    let psqt = if mv.kind() != MoveKind::Promotion {

        Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.to_sq(), phase)
            - Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.from_sq(), phase)
    } else {
        Params::Value::default()
    };

    let mut threat_score = Params::Value::default();
    let mut king_attack_score = Params::Value::default();
    if mv.kind() == MoveKind::None
        && moving_piece.piece_type() != PieceType::King
        && moving_piece.piece_type() != PieceType::Pawn
    {
        let occ_after =
            board.occ() | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
        let attacks_after =
            attacks::piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after);

        let mut threats =
            attacks_after & board.colors(!board.stm()) & !board.pieces(PieceType::King);
        while threats.any() {
            let threat = threats.poplsb();
            threat_score += Params::threat(
                moving_piece.piece_type(),
                board.piece_at(threat).unwrap().piece_type(),
            );
        }

        let opp_king_ring = {
            let king_sq = board.king_sq(!board.stm());
            let attacks = attacks::king_attacks(king_sq);
            (attacks | attacks::pawn_pushes_bb(!board.stm(), attacks))
                & !Bitboard::from_square(king_sq)
        };

        let king_ring_attacks = attacks_after & opp_king_ring;
        if king_ring_attacks.any() {
            king_attack_score +=
                Params::king_ring_attacks(moving_piece.piece_type(), king_ring_attacks.popcount(), phase);
        }
    }

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

    cap_bonus + promo_bonus + pawn_threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
        + king_attack_score
}
