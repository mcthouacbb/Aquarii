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

    fn cap_bonus(moving: PieceType, captured: PieceType) -> Self::Value;
    fn pawn_protected_penalty(pt: PieceType) -> Self::Value;
    fn pawn_threat_evasion(pt: PieceType) -> Self::Value;
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
const CAP_BONUS: [[f32; 5]; 6] = [
    [1.739, 3.172, 3.262, 3.264, 4.775],
    [1.330, 3.052, 2.562, 3.584, 4.749],
    [1.330, 2.163, 2.799, 3.260, 4.777],
    [1.651, 2.403, 2.487, 2.542, 4.335],
    [1.279, 2.000, 2.555, 2.702, 2.407],
    [1.788, 3.083, 2.616, 2.003, 4.830],
];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.082, 2.229, 1.893, 3.070, 3.230];
#[rustfmt::skip]
const PAWN_THREAT_EVASION: [f32; 5] = [0.090, 2.360, 2.121, 2.227, 2.851];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(99.620, 153.703), S(16.908, 139.681), S(44.661, 146.150), S(21.124, 147.078), S(20.065, 138.035), S(9.232, 130.763), S(-8.495, 132.843), S(66.282, 165.032),
        S(24.683, 57.560), S(24.069, 40.065), S(24.526, 53.366), S(41.836, 40.011), S(40.274, 30.772), S(41.240, 31.290), S(38.268, 29.361), S(26.994, 56.489),
        S(-30.558, -23.191), S(5.930, -33.532), S(12.220, -26.307), S(22.103, -25.358), S(31.782, -35.400), S(28.354, -39.308), S(27.386, -42.735), S(-3.878, -30.156),
        S(-35.044, -85.242), S(-19.137, -75.009), S(-2.013, -72.173), S(17.967, -63.325), S(9.685, -65.489), S(4.163, -69.660), S(1.419, -77.920), S(-13.696, -84.183),
        S(-16.240, -107.356), S(-18.748, -79.877), S(3.221, -86.190), S(-12.053, -56.157), S(5.521, -65.521), S(-6.575, -72.183), S(21.239, -84.216), S(13.082, -101.566),
        S(-19.659, -91.392), S(-12.957, -72.397), S(-16.123, -66.582), S(-50.375, -40.197), S(-35.487, -35.969), S(8.602, -59.772), S(29.894, -77.347), S(-3.671, -86.043),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(-29.622, -64.692), S(-31.578, -26.155), S(-58.633, -2.401), S(-32.689, -6.381), S(-24.392, -3.735), S(-30.555, -25.438), S(-51.777, -22.934), S(-24.535, -59.897),
        S(-38.721, -13.272), S(-7.852, -3.595), S(-15.220, 13.519), S(-30.725, 16.010), S(-32.932, 13.841), S(2.424, 3.142), S(-17.073, -4.541), S(-1.623, -29.605),
        S(-5.936, -9.802), S(3.848, 11.524), S(19.675, 18.755), S(8.619, 27.669), S(17.385, 19.835), S(21.053, 15.328), S(25.979, 3.280), S(5.696, -17.976),
        S(1.368, 1.076), S(12.988, 17.779), S(30.138, 25.948), S(42.706, 24.871), S(44.073, 24.295), S(34.963, 24.380), S(27.373, 15.086), S(41.020, -13.980),
        S(-2.557, -8.870), S(10.084, 9.466), S(27.828, 26.677), S(40.283, 21.098), S(45.409, 20.673), S(38.934, 20.137), S(28.874, 11.742), S(12.218, -18.505),
        S(-41.122, -13.166), S(4.720, 3.565), S(34.383, 3.138), S(32.322, 16.029), S(42.411, 16.931), S(36.590, 0.955), S(32.569, -7.393), S(-29.262, -9.661),
        S(-43.424, -25.824), S(-23.940, -15.556), S(-3.496, -2.338), S(-1.912, 4.282), S(4.640, 5.153), S(14.070, -2.061), S(-4.250, -18.923), S(-22.833, -22.891),
        S(-78.309, -26.158), S(-55.399, -43.804), S(-49.289, -13.411), S(-26.636, -17.306), S(-29.498, -20.535), S(-13.028, -23.346), S(-48.244, -42.145), S(-72.856, -18.310),
    ],
    [
        S(-9.104, -16.863), S(-58.766, -0.947), S(5.152, -18.495), S(-32.760, 1.384), S(-31.532, -4.202), S(-4.896, -14.212), S(-47.248, -5.130), S(1.930, -21.691),
        S(-30.387, -13.360), S(-11.758, -1.076), S(-21.783, 1.817), S(-54.125, 3.786), S(-46.205, 4.882), S(-26.905, -0.441), S(-17.771, 0.903), S(-17.694, -22.409),
        S(-22.055, -1.517), S(-0.128, 3.390), S(1.824, 7.704), S(11.406, 1.612), S(2.936, 5.213), S(6.599, 10.060), S(17.927, 0.317), S(-3.487, -1.993),
        S(-8.445, -3.929), S(3.465, 9.204), S(12.231, 12.348), S(11.457, 16.533), S(17.596, 15.517), S(14.390, 10.528), S(13.768, 7.459), S(1.239, -9.531),
        S(2.534, -9.394), S(-2.868, 11.056), S(1.432, 17.533), S(28.160, 15.039), S(22.843, 13.169), S(9.713, 14.310), S(5.848, 4.624), S(16.015, -18.665),
        S(3.029, -8.164), S(17.580, 0.939), S(17.521, 13.060), S(8.418, 13.329), S(10.601, 18.208), S(12.090, 12.193), S(24.107, -3.073), S(6.977, -9.752),
        S(-10.166, -3.293), S(18.356, -11.589), S(13.476, -2.974), S(-9.623, 5.656), S(0.721, 4.735), S(25.514, -6.303), S(34.192, -12.347), S(1.280, -14.296),
        S(-16.827, -19.599), S(-18.557, -2.069), S(-15.436, -27.491), S(-31.297, -2.879), S(-15.527, -11.637), S(-17.922, -16.282), S(-8.760, -10.953), S(-21.873, -26.334),
    ],
    [
        S(13.253, 18.522), S(-7.158, 19.823), S(-30.917, 28.870), S(-31.220, 22.027), S(-22.161, 15.895), S(-21.292, 14.689), S(-0.529, 12.180), S(44.311, 5.573),
        S(-13.382, 18.333), S(-11.716, 20.166), S(-10.433, 21.599), S(-5.159, 12.023), S(-6.126, 6.652), S(-10.225, 6.479), S(-7.481, 6.661), S(23.529, 1.397),
        S(-13.670, 14.049), S(-10.105, 10.795), S(-6.494, 5.698), S(0.440, -2.753), S(6.695, -12.449), S(3.801, -9.349), S(16.760, -7.047), S(15.255, -6.465),
        S(-13.040, 10.849), S(-7.708, 4.314), S(3.149, 2.771), S(4.155, -3.738), S(13.868, -15.396), S(13.420, -15.243), S(18.356, -13.523), S(23.483, -12.879),
        S(-16.461, 6.714), S(-18.437, 4.502), S(-1.315, -0.899), S(12.853, -8.299), S(15.709, -13.098), S(1.077, -7.973), S(15.528, -14.484), S(7.471, -11.886),
        S(-27.588, 4.137), S(-8.827, -5.609), S(-1.778, -5.594), S(3.818, -9.009), S(12.124, -13.626), S(7.192, -16.318), S(33.126, -26.285), S(14.431, -21.068),
        S(-40.642, 3.416), S(-23.599, -1.907), S(-1.257, -7.531), S(1.775, -10.434), S(2.573, -14.479), S(3.844, -16.530), S(19.634, -24.753), S(-14.981, -11.539),
        S(2.699, -8.855), S(-10.334, 1.742), S(2.123, 3.438), S(16.538, -4.058), S(15.407, -10.829), S(10.496, -15.562), S(20.183, -16.731), S(9.751, -19.755),
    ],
    [
        S(-11.955, 0.098), S(-28.617, 13.072), S(-46.687, 32.148), S(-34.453, 22.739), S(-32.803, 22.322), S(-51.085, 25.365), S(-11.484, 1.950), S(2.491, -1.686),
        S(-8.336, -13.971), S(-20.046, 9.078), S(-27.027, 22.236), S(-57.164, 41.343), S(-66.704, 54.319), S(-35.996, 27.873), S(-23.586, 15.493), S(4.587, -2.890),
        S(-4.379, -17.541), S(4.695, -8.584), S(-13.114, 16.349), S(-6.209, 11.132), S(-9.878, 23.173), S(-1.811, 17.508), S(22.391, -2.927), S(11.335, -4.650),
        S(-13.852, -3.945), S(-1.858, 0.748), S(-6.326, 9.207), S(-1.231, 16.205), S(5.005, 18.090), S(4.679, 19.006), S(14.655, 9.479), S(17.315, -5.538),
        S(2.802, -15.963), S(-7.297, 3.254), S(7.724, 1.494), S(18.019, 5.488), S(17.695, 8.774), S(15.196, 3.154), S(11.968, 3.968), S(19.767, -12.425),
        S(-7.271, -12.492), S(8.642, -10.759), S(19.578, -6.577), S(10.709, 0.986), S(19.460, -0.564), S(22.571, -3.800), S(35.532, -18.533), S(17.547, -20.404),
        S(-13.763, -14.969), S(3.423, -14.783), S(17.766, -24.395), S(12.418, -12.771), S(15.493, -14.227), S(26.620, -28.416), S(30.133, -36.152), S(15.909, -40.805),
        S(-2.677, -21.249), S(-33.928, -1.985), S(-13.875, -9.692), S(18.331, -65.281), S(-0.984, -22.030), S(-16.362, -6.525), S(-3.470, -19.428), S(-15.680, -22.877),
    ],
    [
        S(22.868, -34.611), S(37.783, -19.730), S(27.697, -13.180), S(-23.788, -0.178), S(-12.777, 1.248), S(15.254, 3.437), S(31.672, 0.265), S(54.747, -32.142),
        S(-5.798, -12.416), S(-2.678, 19.591), S(-24.164, 19.374), S(-10.675, 17.935), S(-42.750, 31.188), S(-8.392, 34.840), S(21.674, 25.643), S(-6.169, 7.465),
        S(-57.238, 2.070), S(-4.746, 22.478), S(-46.327, 34.898), S(-68.518, 47.399), S(-50.388, 49.852), S(1.929, 43.640), S(6.583, 35.524), S(-15.091, 12.040),
        S(-60.949, 1.633), S(-49.384, 25.743), S(-72.185, 41.224), S(-121.348, 55.924), S(-104.356, 55.670), S(-86.086, 51.128), S(-67.876, 36.999), S(-86.423, 14.721),
        S(-59.875, -9.034), S(-60.068, 16.469), S(-74.058, 32.670), S(-108.931, 50.125), S(-110.581, 50.107), S(-77.944, 37.464), S(-70.052, 22.802), S(-91.910, 6.258),
        S(-17.841, -20.917), S(0.519, -1.549), S(-42.588, 17.376), S(-61.907, 30.071), S(-60.498, 30.430), S(-47.145, 19.888), S(-14.040, 2.858), S(-16.522, -16.409),
        S(42.527, -39.372), S(20.305, -17.240), S(5.236, -3.834), S(-30.339, 5.304), S(-30.462, 7.937), S(-1.246, -3.506), S(37.088, -20.930), S(34.804, -38.833),
        S(25.828, -66.225), S(67.284, -56.707), S(46.120, -42.747), S(-18.140, -35.143), S(39.772, -54.860), S(2.995, -40.510), S(62.715, -55.654), S(39.985, -76.681),
    ],
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.725, 1.154, 1.109, 0.456, 1.011],
    [0.154, -0.063, 0.782, 0.719, 0.792],
    [0.107, 0.612, -0.077, 0.735, 0.745],
    [0.264, 0.612, 0.611, -0.074, 1.135],
    [0.162, 0.411, 0.123, 0.115, 0.108],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.082, -1.890];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.532;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.549;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(moving: PieceType, captured: PieceType) -> Self::Value {
        CAP_BONUS[moving as usize][captured as usize]
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

pub fn get_policy(board: &Board, mv: Move) -> f32 {
    get_policy_impl::<PolicyParams>(board, mv)
}

pub fn get_policy_impl<Params: PolicyValues>(board: &Board, mv: Move) -> Params::Value {
    let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let pawn_protected = attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let captured_piece = board.piece_at(mv.to_sq());
    let cap_bonus = if let Some(captured) = captured_piece {
        Params::cap_bonus(moving_piece.piece_type(), captured.piece_type())
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

    cap_bonus + promo_bonus + pawn_threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
