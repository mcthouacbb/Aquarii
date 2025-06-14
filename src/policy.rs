use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    types::{Color, Piece, PieceType, Square},
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
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn good_see_bonus() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

const CAP_BONUS: [f32; 5] = [1.522, 1.985, 2.058, 2.159, 2.720];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [0.625, 2.073, 1.847, 2.796, 3.076];
const PAWN_THREAT_EVASION: [f32; 5] = [0.368, 2.448, 2.111, 2.245, 2.858];
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(95.829, 152.922), S(61.080, 133.404), S(76.721, 146.252), S(65.149, 139.797), S(78.499, 125.170), S(31.222, 123.228), S(28.842, 120.666), S(68.687, 155.491),
        S(26.704, 57.259), S(25.934, 41.710), S(45.623, 54.227), S(63.871, 38.906), S(61.783, 27.073), S(62.468, 24.375), S(40.153, 23.619), S(32.941, 47.828),
        S(-38.064, -18.593), S(17.188, -32.084), S(11.778, -19.521), S(37.330, -25.333), S(44.655, -37.723), S(29.312, -40.970), S(38.578, -48.982), S(-10.560, -33.924),
        S(-41.188, -77.931), S(-23.413, -66.435), S(-7.046, -64.201), S(12.161, -57.070), S(4.505, -63.171), S(1.149, -68.557), S(-1.832, -78.657), S(-19.278, -85.976),
        S(-17.701, -98.731), S(-26.745, -69.020), S(-2.004, -78.779), S(-19.667, -51.989), S(-4.094, -63.784), S(-9.197, -71.763), S(12.757, -82.239), S(11.751, -101.810),
        S(-19.982, -80.557), S(-16.516, -62.533), S(-21.207, -59.379), S(-55.337, -38.315), S(-41.671, -37.833), S(4.832, -59.634), S(24.450, -75.161), S(-4.244, -83.661),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(7.309, -71.454), S(37.222, -38.110), S(6.545, -12.805), S(50.194, -20.327), S(46.231, -15.900), S(44.552, -34.156), S(28.493, -37.711), S(22.454, -68.147),
        S(-26.863, -16.530), S(13.463, -6.301), S(11.687, 9.791), S(26.864, 7.481), S(26.979, 2.959), S(43.360, -3.959), S(10.816, -7.333), S(16.668, -31.441),
        S(4.940, -10.828), S(35.002, 6.306), S(62.846, 11.005), S(37.564, 25.066), S(63.682, 12.944), S(58.388, 8.524), S(58.068, -3.269), S(21.810, -21.558),
        S(2.295, 0.972), S(13.851, 20.035), S(39.226, 27.847), S(46.815, 27.490), S(48.464, 26.895), S(52.356, 23.125), S(28.156, 17.602), S(43.064, -13.189),
        S(-8.926, -4.918), S(15.252, 12.019), S(24.883, 30.585), S(37.796, 26.428), S(43.349, 24.432), S(37.310, 24.865), S(30.974, 13.425), S(6.663, -15.630),
        S(-47.467, -10.827), S(-2.989, 8.262), S(27.666, 7.135), S(28.191, 19.971), S(38.440, 20.375), S(29.647, 3.203), S(25.473, -4.047), S(-35.762, -7.853),
        S(-53.227, -22.657), S(-35.683, -11.146), S(-12.194, 2.605), S(-11.047, 8.067), S(-4.648, 8.063), S(5.784, 1.537), S(-15.509, -15.550), S(-32.142, -21.690),
        S(-89.368, -24.524), S(-59.934, -43.066), S(-63.113, -9.599), S(-37.929, -13.353), S(-42.358, -17.326), S(-27.378, -19.624), S(-52.802, -41.360), S(-85.852, -16.135),
    ],
    [
        S(4.731, -21.561), S(-51.214, -3.492), S(33.205, -24.493), S(1.023, -5.242), S(-11.403, -8.253), S(30.083, -21.661), S(-37.026, -7.354), S(-0.307, -22.008),
        S(-38.049, -13.052), S(6.937, -5.388), S(-12.834, 0.799), S(-26.754, 0.359), S(-2.121, -4.032), S(-11.282, -2.723), S(8.018, -4.077), S(-18.498, -22.916),
        S(-29.678, -1.466), S(2.303, 2.564), S(20.380, 4.051), S(24.665, -0.318), S(15.449, 4.947), S(34.889, 3.931), S(18.216, 0.751), S(-13.637, -1.673),
        S(-13.016, -3.835), S(5.265, 8.154), S(15.187, 12.659), S(22.112, 15.535), S(24.747, 15.519), S(18.828, 11.203), S(16.962, 6.157), S(-6.285, -8.782),
        S(-1.107, -9.415), S(-2.020, 11.643), S(0.468, 18.925), S(29.132, 16.603), S(27.840, 13.567), S(6.794, 15.411), S(4.075, 5.572), S(13.376, -19.194),
        S(-3.168, -5.864), S(12.831, 2.970), S(13.513, 14.803), S(5.600, 15.476), S(4.143, 20.721), S(10.366, 13.801), S(17.026, -1.029), S(2.948, -9.138),
        S(-17.561, -2.491), S(10.145, -8.092), S(6.319, -0.544), S(-18.649, 8.307), S(-5.289, 6.668), S(16.136, -3.704), S(29.298, -10.518), S(-6.906, -13.478),
        S(-31.679, -18.095), S(-26.469, -0.869), S(-23.053, -25.891), S(-40.682, -0.732), S(-28.257, -8.891), S(-22.708, -15.311), S(-20.681, -9.187), S(-32.454, -25.472),
    ],
    [
        S(20.487, 14.243), S(0.218, 16.726), S(-20.733, 26.286), S(-18.513, 18.937), S(-7.229, 12.535), S(-7.010, 11.149), S(10.155, 8.994), S(47.332, 2.759),
        S(-3.970, 16.715), S(2.854, 18.501), S(15.102, 18.358), S(23.804, 8.934), S(19.309, 4.378), S(19.864, 2.623), S(12.165, 4.435), S(33.791, 0.269),
        S(-3.491, 14.649), S(8.008, 10.533), S(19.967, 4.938), S(28.881, -2.849), S(36.006, -12.613), S(32.728, -9.449), S(39.511, -6.963), S(29.493, -6.656),
        S(-8.527, 13.234), S(3.547, 6.337), S(18.423, 4.691), S(20.974, -1.036), S(30.475, -11.437), S(27.740, -10.962), S(31.189, -10.188), S(29.507, -9.986),
        S(-15.111, 9.036), S(-16.030, 8.290), S(7.556, 1.451), S(18.314, -4.057), S(21.375, -8.527), S(9.005, -3.780), S(21.337, -11.155), S(10.904, -9.802),
        S(-36.430, 7.437), S(-12.108, -3.034), S(-4.072, -2.560), S(-1.650, -4.370), S(8.397, -8.897), S(2.027, -11.191), S(28.849, -22.260), S(7.682, -18.399),
        S(-50.460, 4.311), S(-31.874, 0.131), S(-9.807, -4.560), S(-8.590, -6.254), S(-6.573, -10.070), S(-6.635, -11.669), S(9.908, -21.550), S(-23.652, -10.354),
        S(-3.608, -10.973), S(-20.310, 1.518), S(-9.166, 4.518), S(6.830, -3.036), S(4.556, -9.099), S(2.085, -15.592), S(11.370, -17.426), S(5.284, -22.627),
    ],
    [
        S(-13.981, 0.024), S(-25.281, 11.252), S(-34.828, 27.216), S(-30.857, 22.088), S(-21.946, 18.830), S(-35.012, 19.529), S(-6.004, -0.098), S(0.236, -1.090),
        S(-11.430, -13.897), S(-13.335, 6.131), S(-19.735, 20.700), S(-43.564, 38.046), S(-52.030, 50.792), S(-24.475, 25.035), S(-12.184, 12.255), S(3.897, -2.910),
        S(-5.796, -18.142), S(10.757, -10.498), S(-3.315, 13.781), S(9.237, 7.960), S(5.509, 19.815), S(13.899, 13.561), S(31.132, -4.518), S(11.446, -5.657),
        S(-16.171, -3.187), S(5.912, -2.399), S(0.530, 8.711), S(7.282, 15.642), S(15.577, 16.614), S(13.634, 17.085), S(22.391, 7.256), S(17.279, -4.825),
        S(3.202, -17.398), S(-7.663, 3.995), S(12.333, 0.850), S(20.775, 6.539), S(21.448, 9.541), S(21.168, 2.911), S(10.721, 6.410), S(19.244, -12.916),
        S(-12.017, -11.492), S(4.683, -7.726), S(17.149, -4.682), S(7.092, 4.935), S(16.599, 2.360), S(20.654, -2.012), S(32.255, -16.098), S(14.176, -18.983),
        S(-23.687, -12.326), S(-4.942, -11.160), S(10.372, -19.872), S(4.295, -7.308), S(7.344, -8.607), S(19.606, -24.555), S(22.640, -33.167), S(7.511, -39.216),
        S(-18.756, -16.157), S(-45.632, 1.984), S(-26.014, -5.377), S(10.264, -61.821), S(-11.513, -17.999), S(-28.261, -2.233), S(-13.641, -16.993), S(-29.846, -18.981),
    ],
    [
        S(-0.759, -37.578), S(0.222, -18.051), S(-9.485, -11.338), S(-62.353, 2.280), S(-48.897, 3.403), S(-22.938, 5.726), S(-3.141, 2.782), S(25.551, -32.648),
        S(-44.154, -8.821), S(-36.238, 22.848), S(-56.117, 22.108), S(-40.692, 20.719), S(-70.812, 33.665), S(-36.077, 37.552), S(-5.958, 28.287), S(-37.497, 10.367),
        S(-88.089, 4.800), S(-32.128, 25.193), S(-72.033, 37.141), S(-90.757, 49.186), S(-72.112, 51.794), S(-19.814, 45.671), S(-15.583, 37.803), S(-43.758, 15.144),
        S(-84.320, 3.564), S(-69.676, 27.218), S(-90.131, 42.723), S(-137.623, 57.429), S(-119.784, 56.948), S(-102.968, 52.762), S(-86.375, 38.648), S(-107.356, 16.650),
        S(-75.822, -8.537), S(-72.745, 17.052), S(-84.348, 32.817), S(-117.966, 50.422), S(-118.677, 50.503), S(-85.542, 37.850), S(-79.830, 23.336), S(-104.082, 6.951),
        S(-26.371, -22.343), S(-5.044, -2.750), S(-45.268, 16.025), S(-62.352, 28.626), S(-59.748, 29.061), S(-49.190, 19.001), S(-17.356, 1.965), S(-23.120, -17.047),
        S(37.507, -41.767), S(18.412, -19.583), S(6.505, -6.191), S(-23.162, 1.476), S(-23.351, 4.669), S(5.002, -6.564), S(34.918, -21.415), S(31.731, -39.604),
        S(17.150, -71.502), S(68.148, -60.003), S(53.008, -46.533), S(-9.776, -39.892), S(53.750, -58.814), S(9.237, -44.247), S(66.319, -56.791), S(33.806, -78.122),
    ],
];
const PROMO_BONUS: [f32; 2] = [1.042, -1.857];
const BAD_SEE_PENALTY: f32 = -2.449;
const GOOD_SEE_BONUS: f32 = 1.272;
const CHECK_BONUS: f32 = 0.512;

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

    fn promo_bonus(pt: PieceType) -> Self::Value {
        match pt {
            PieceType::Queen => PROMO_BONUS[0],
            _ => PROMO_BONUS[1],
        }
    }

    fn bad_see_penalty() -> Self::Value {
        BAD_SEE_PENALTY
    }

    fn good_see_bonus() -> Self::Value {
        GOOD_SEE_BONUS
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
    let psqt = if mv.kind() != MoveKind::Promotion {
        let phase = (4 * board.pieces(PieceType::Queen).popcount()
            + 2 * board.pieces(PieceType::Rook).popcount()
            + board.pieces(PieceType::Bishop).popcount()
            + board.pieces(PieceType::Knight).popcount()) as i32;

        Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.to_sq(), phase)
            - Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.from_sq(), phase)
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

    let good_see = see::see(board, mv, 350) && mv.kind() == MoveKind::None;
    let good_see_bonus = if good_see {
        Params::good_see_bonus()
    } else {
        Params::Value::default()
    };

    let check_bonus = if board.gives_direct_check(mv) {
        Params::check_bonus()
    } else {
        Params::Value::default()
    };

    cap_bonus + promo_bonus + pawn_threat_evasion + bad_see_penalty + good_see_bonus + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
}
