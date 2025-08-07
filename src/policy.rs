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
    fn threat_evasion(pt: PieceType, threat: PieceType) -> Self::Value;
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
const CAP_BONUS: [f32; 5] = [1.518, 2.568, 2.750, 2.713, 3.453];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.134, 2.273, 1.957, 3.078, 3.220];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 6] = [
    [0.625, 2.418, 2.299, 2.157, 2.960],
    [0.302, 0.538, 1.378, 1.885, 2.505],
    [0.191, 0.796, 0.602, 2.004, 2.554],
    [0.249, 0.694, 0.726, 0.791, 2.509],
    [0.120, 0.433, 0.577, 0.531, 0.902],
    [0.687, 0.791, 0.673, 1.019, 0.000],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(126.811, 145.875), S(56.620, 137.956), S(88.879, 146.474), S(71.446, 151.542), S(70.654, 141.075), S(62.321, 131.394), S(38.508, 131.493), S(100.580, 158.396),
        S(39.725, 51.611), S(36.694, 42.409), S(50.093, 56.526), S(66.831, 49.538), S(65.993, 39.164), S(68.954, 34.462), S(54.835, 31.657), S(45.051, 51.370),
        S(-27.762, -29.225), S(6.097, -30.661), S(19.986, -20.600), S(33.434, -15.566), S(42.585, -25.891), S(36.281, -35.057), S(30.388, -40.657), S(-2.622, -34.236),
        S(-41.924, -90.941), S(-24.670, -74.938), S(-6.673, -67.914), S(17.056, -54.222), S(7.428, -57.080), S(0.469, -67.175), S(-2.788, -78.416), S(-20.551, -89.000),
        S(-25.906, -114.422), S(-28.445, -80.980), S(-4.351, -84.579), S(-16.584, -50.190), S(-0.866, -60.282), S(-13.378, -71.556), S(11.430, -85.530), S(2.646, -107.702),
        S(-33.241, -100.091), S(-27.568, -75.096), S(-27.823, -66.480), S(-58.682, -37.768), S(-45.095, -33.904), S(-3.178, -60.789), S(15.407, -79.909), S(-17.758, -94.004),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(11.667, -75.626), S(13.274, -36.480), S(-37.786, -5.995), S(40.319, -21.391), S(21.463, -12.926), S(-4.187, -26.835), S(10.596, -35.254), S(23.189, -68.084),
        S(-34.142, -19.579), S(-2.376, -6.534), S(-4.073, 10.777), S(-9.033, 12.420), S(-10.637, 9.271), S(25.934, -1.798), S(-6.559, -7.248), S(11.327, -36.564),
        S(-5.049, -13.135), S(10.248, 9.609), S(36.337, 15.782), S(16.971, 28.314), S(26.292, 20.709), S(40.343, 11.896), S(31.355, 2.082), S(13.146, -23.806),
        S(4.476, -2.883), S(8.243, 20.038), S(33.153, 27.453), S(43.454, 27.749), S(46.037, 28.220), S(39.325, 25.940), S(24.585, 16.269), S(42.729, -15.989),
        S(-4.596, -10.469), S(11.149, 8.122), S(24.663, 28.751), S(40.177, 24.224), S(44.955, 23.477), S(36.152, 22.615), S(29.517, 10.810), S(10.614, -20.435),
        S(-41.553, -14.128), S(2.634, 3.230), S(31.809, 4.980), S(30.322, 18.037), S(39.830, 18.852), S(34.822, 2.118), S(30.984, -7.952), S(-29.293, -10.793),
        S(-48.761, -25.941), S(-28.496, -14.911), S(-8.325, -0.869), S(-6.105, 5.904), S(1.324, 6.400), S(9.779, -0.651), S(-9.026, -19.343), S(-28.042, -22.558),
        S(-81.153, -27.327), S(-62.499, -42.575), S(-54.094, -12.463), S(-31.804, -15.576), S(-36.877, -18.768), S(-18.098, -22.577), S(-54.838, -40.745), S(-76.020, -18.290),
    ],
    [
        S(9.397, -23.727), S(-55.308, -3.042), S(19.674, -22.561), S(5.812, -6.430), S(-15.444, -8.043), S(14.546, -18.511), S(-38.212, -6.669), S(13.894, -26.580),
        S(-33.915, -16.184), S(-8.477, -2.546), S(-21.166, 2.277), S(-44.084, 1.779), S(-40.773, 4.509), S(-18.937, -1.646), S(-8.800, -0.190), S(-14.580, -26.002),
        S(-26.325, -2.117), S(1.819, 2.850), S(-1.759, 9.402), S(16.615, 2.655), S(7.009, 6.216), S(2.047, 12.987), S(18.953, 0.018), S(-7.612, -3.502),
        S(-11.252, -5.767), S(2.213, 10.261), S(12.154, 13.669), S(14.489, 18.440), S(20.320, 18.342), S(13.977, 13.172), S(13.343, 8.780), S(2.273, -13.106),
        S(-0.369, -11.146), S(-2.684, 11.206), S(0.745, 18.462), S(30.793, 16.680), S(22.919, 15.261), S(9.190, 15.461), S(7.439, 4.926), S(13.728, -20.059),
        S(3.621, -9.183), S(16.079, 0.548), S(19.223, 12.907), S(8.991, 14.023), S(10.625, 19.967), S(14.076, 12.308), S(22.694, -2.296), S(7.565, -11.510),
        S(-15.374, -4.100), S(18.321, -12.148), S(9.360, -2.860), S(-10.810, 5.784), S(-1.100, 4.961), S(23.629, -6.016), S(33.428, -12.577), S(-4.738, -15.304),
        S(-17.122, -21.322), S(-23.403, -2.317), S(-20.056, -27.211), S(-33.645, -2.543), S(-19.635, -11.115), S(-22.898, -15.499), S(-13.697, -11.824), S(-22.832, -29.332),
    ],
    [
        S(12.995, 17.345), S(-5.144, 19.265), S(-29.950, 29.025), S(-26.011, 21.191), S(-16.937, 15.311), S(-18.423, 14.447), S(1.173, 12.028), S(42.900, 4.978),
        S(-8.447, 17.183), S(-5.051, 19.755), S(-0.146, 20.914), S(3.342, 12.352), S(4.319, 6.907), S(2.988, 6.890), S(2.341, 7.415), S(29.852, 0.328),
        S(-10.688, 14.147), S(-4.631, 11.742), S(0.741, 7.134), S(8.655, -1.657), S(15.365, -10.267), S(9.067, -6.021), S(24.987, -5.737), S(20.273, -5.815),
        S(-11.616, 11.015), S(-4.036, 5.503), S(7.648, 4.863), S(10.135, -1.061), S(20.021, -12.149), S(17.579, -11.843), S(23.034, -11.207), S(24.270, -12.089),
        S(-16.645, 6.719), S(-17.103, 4.991), S(-0.059, 0.484), S(15.349, -6.836), S(16.681, -10.410), S(4.574, -6.477), S(16.153, -12.987), S(8.504, -11.763),
        S(-27.252, 3.171), S(-9.424, -5.404), S(-1.600, -5.094), S(4.024, -8.257), S(12.221, -13.271), S(7.017, -15.425), S(33.383, -25.968), S(14.020, -21.951),
        S(-42.839, 2.375), S(-25.473, -2.715), S(-2.677, -7.525), S(0.384, -10.616), S(1.254, -14.666), S(1.798, -16.017), S(18.078, -24.933), S(-18.517, -12.087),
        S(-3.178, -9.315), S(-13.176, 0.855), S(0.263, 2.721), S(14.078, -4.240), S(13.162, -11.246), S(7.391, -15.899), S(16.305, -17.609), S(2.865, -20.087),
    ],
    [
        S(-6.656, -2.187), S(-22.755, 11.363), S(-40.912, 30.580), S(-48.076, 29.531), S(-33.714, 23.986), S(-41.676, 23.752), S(-5.603, 0.500), S(7.605, -3.427),
        S(-11.613, -13.290), S(-12.938, 6.989), S(-23.368, 21.906), S(-57.431, 43.382), S(-67.935, 56.143), S(-29.475, 26.637), S(-15.300, 12.921), S(8.837, -4.764),
        S(-3.576, -19.983), S(11.209, -11.294), S(-6.916, 15.175), S(3.439, 9.715), S(2.148, 20.444), S(5.899, 17.152), S(34.096, -7.157), S(16.004, -7.167),
        S(-16.611, -3.071), S(5.590, -2.256), S(-1.239, 8.567), S(8.031, 14.652), S(14.821, 16.358), S(11.815, 17.759), S(21.934, 7.441), S(19.943, -7.200),
        S(4.114, -19.143), S(-8.063, 5.006), S(12.356, 1.055), S(23.149, 4.240), S(23.399, 8.410), S(19.917, 3.618), S(14.895, 3.387), S(16.710, -11.866),
        S(-11.783, -11.428), S(9.867, -11.415), S(19.438, -5.389), S(12.726, 0.974), S(19.324, 1.473), S(22.021, -2.581), S(33.887, -15.547), S(13.212, -19.023),
        S(-20.523, -13.543), S(-0.343, -13.566), S(14.163, -22.726), S(9.211, -10.616), S(12.115, -12.794), S(23.106, -27.178), S(24.950, -32.260), S(8.095, -38.569),
        S(-15.098, -17.791), S(-38.961, -1.858), S(-19.484, -9.121), S(10.986, -59.412), S(-6.641, -20.703), S(-23.707, -4.013), S(-10.811, -18.023), S(-26.437, -20.058),
    ],
    [
        S(29.899, -35.133), S(43.915, -18.204), S(38.538, -13.351), S(-12.754, 0.179), S(-0.181, 1.532), S(23.483, 4.231), S(43.045, 1.072), S(66.805, -33.371),
        S(1.189, -9.978), S(5.213, 20.752), S(-17.124, 20.498), S(-0.219, 18.569), S(-32.042, 31.836), S(4.736, 35.225), S(33.330, 26.462), S(4.428, 9.473),
        S(-46.102, 3.049), S(3.483, 23.455), S(-36.118, 34.971), S(-57.324, 47.206), S(-40.290, 49.876), S(10.792, 43.810), S(15.501, 36.357), S(-8.651, 14.417),
        S(-55.038, 3.160), S(-42.646, 26.295), S(-64.137, 41.378), S(-112.247, 55.987), S(-97.321, 55.700), S(-77.571, 51.221), S(-61.757, 37.859), S(-80.019, 16.932),
        S(-52.005, -8.380), S(-53.176, 16.879), S(-66.029, 32.383), S(-100.852, 49.467), S(-103.929, 49.796), S(-72.375, 37.662), S(-67.892, 24.124), S(-91.009, 8.625),
        S(-15.463, -20.343), S(4.392, -1.610), S(-37.210, 16.854), S(-54.302, 29.095), S(-53.913, 29.466), S(-44.622, 20.000), S(-15.791, 3.736), S(-22.171, -13.741),
        S(39.438, -38.153), S(20.059, -17.003), S(8.914, -4.441), S(-23.840, 4.445), S(-23.899, 6.994), S(-0.972, -3.420), S(27.498, -18.832), S(20.905, -35.225),
        S(-7.101, -63.455), S(65.828, -57.432), S(43.741, -41.928), S(-12.447, -35.170), S(44.114, -51.776), S(3.713, -40.015), S(45.052, -51.865), S(11.269, -70.326),
    ],
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.717, 1.002, 1.070, 0.460, 0.987],
    [0.176, 0.104, 0.720, 0.568, 0.735],
    [0.043, 0.683, -0.039, 0.665, 0.763],
    [0.213, 0.498, 0.559, 0.033, 1.126],
    [0.016, 0.307, -0.018, 0.007, 0.280],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.111, -1.829];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.452;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.557;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        CAP_BONUS[pt as usize]
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        PAWN_PROTECTED_PENALTY[pt as usize]
    }

    fn threat_evasion(pt: PieceType, threat: PieceType) -> Self::Value {
        THREAT_EVASION[threat as usize][pt as usize]
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
    attacked: [Bitboard; 2],
    attacked_by: [[Bitboard; 6]; 2],
}

impl PolicyData {
    pub fn new(board: &Board) -> Self {
        let mut result: PolicyData = Self {
            attacked: [Bitboard::NONE; 2],
            attacked_by: [[Bitboard::NONE; 6]; 2],
        };

        result.add_attacks(
            Color::White,
            PieceType::Pawn,
            attacks::pawn_attacks_bb(Color::White, board.colored_pieces(Piece::WhitePawn)),
        );
        result.add_attacks(
            Color::Black,
            PieceType::Pawn,
            attacks::pawn_attacks_bb(Color::Black, board.colored_pieces(Piece::BlackPawn)),
        );

        result.add_attacks(
            Color::White,
            PieceType::King,
            attacks::king_attacks(board.king_sq(Color::White)),
        );
        result.add_attacks(
            Color::Black,
            PieceType::King,
            attacks::king_attacks(board.king_sq(Color::Black)),
        );

        for c in [Color::White, Color::Black] {
            for pt in [
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
            ] {
                let mut bb = board.colored_pieces(Piece::new(c, pt));
                while bb.any() {
                    let sq = bb.poplsb();
                    let attacks = attacks::piece_attacks(pt, sq, board.occ());
                    result.add_attacks(c, pt, attacks);
                }
            }
        }

        result
    }

    fn add_attacks(&mut self, c: Color, pt: PieceType, attacks: Bitboard) {
        self.attacked[c as usize] |= attacks;
        self.attacked_by[c as usize][pt as usize] |= attacks;
    }

    fn attacked_by(&self, c: Color, pt: PieceType) -> Bitboard {
        self.attacked_by[c as usize][pt as usize]
    }
}

pub fn get_policy(board: &Board, policy_data: PolicyData, mv: Move) -> f32 {
    get_policy_impl::<PolicyParams>(board, policy_data, mv)
}

pub fn get_policy_impl<Params: PolicyValues>(
    board: &Board,
    policy_data: PolicyData,
    mv: Move,
) -> Params::Value {
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

    let mut threat_evasion = Params::Value::default();

    if moving_piece.piece_type() != PieceType::King
        && policy_data.attacked[!board.stm() as usize].has(mv.from_sq())
        && !policy_data.attacked[!board.stm() as usize].has(mv.to_sq())
    {
        for pt in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ] {
            if policy_data.attacked_by(!board.stm(), pt).has(mv.from_sq()) {
                threat_evasion += Params::threat_evasion(moving_piece.piece_type(), pt);
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
