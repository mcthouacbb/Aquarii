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

const CAP_BONUS: [f32; 5] = [1.525, 2.569, 2.731, 2.651, 3.246];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.106, 2.186, 1.931, 3.086, 3.397];
const THREAT_EVASION: [[f32; 5]; 6] = [
    [0.242, 2.351, 2.047, 2.260, 2.745],
    [0.000, 0.000, 0.000, 0.000, 0.000],
    [0.000, 0.000, 0.000, 0.000, 0.000],
    [0.000, 0.000, 0.000, 0.000, 0.000],
    [0.000, 0.000, 0.000, 0.000, 0.000],
    [0.000, 0.000, 0.000, 0.000, 0.000],
];
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(110.840, 146.405),
        S(37.720, 137.033),
        S(69.373, 144.603),
        S(51.710, 149.154),
        S(48.300, 140.846),
        S(35.372, 130.736),
        S(14.559, 132.039),
        S(81.767, 157.854),
        S(30.691, 51.280),
        S(31.473, 39.515),
        S(41.667, 53.028),
        S(55.689, 45.931),
        S(55.518, 36.532),
        S(59.135, 32.418),
        S(46.659, 30.007),
        S(34.301, 51.475),
        S(-32.104, -27.950),
        S(7.130, -33.610),
        S(17.375, -23.940),
        S(27.486, -18.330),
        S(37.514, -28.427),
        S(32.220, -36.050),
        S(27.612, -41.439),
        S(-7.314, -32.696),
        S(-40.658, -88.825),
        S(-22.806, -75.706),
        S(-3.980, -70.222),
        S(17.074, -56.910),
        S(8.232, -59.189),
        S(1.056, -66.681),
        S(-3.028, -77.787),
        S(-21.113, -86.376),
        S(-22.757, -111.123),
        S(-24.695, -80.776),
        S(0.180, -85.626),
        S(-14.133, -51.106),
        S(1.844, -60.814),
        S(-10.531, -70.451),
        S(13.398, -84.030),
        S(4.392, -103.640),
        S(-26.053, -95.285),
        S(-19.377, -73.983),
        S(-19.527, -66.623),
        S(-52.244, -37.252),
        S(-38.476, -33.239),
        S(3.689, -58.409),
        S(21.651, -77.177),
        S(-12.223, -88.153),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
        S(0.000, 0.000),
    ],
    [
        S(8.401, -72.714),
        S(-0.199, -31.341),
        S(-54.134, -1.952),
        S(12.875, -14.936),
        S(2.585, -9.162),
        S(-25.983, -23.747),
        S(-14.018, -28.321),
        S(14.057, -65.001),
        S(-40.896, -14.471),
        S(-11.054, -3.616),
        S(-18.817, 13.535),
        S(-22.723, 13.847),
        S(-25.753, 11.898),
        S(-0.905, 3.402),
        S(-21.359, -4.012),
        S(-2.335, -29.648),
        S(-8.363, -10.489),
        S(4.876, 10.515),
        S(25.974, 16.414),
        S(5.475, 27.775),
        S(13.976, 20.088),
        S(30.097, 11.421),
        S(26.338, 2.188),
        S(3.624, -19.046),
        S(2.583, -0.673),
        S(10.964, 18.080),
        S(28.273, 25.633),
        S(40.513, 24.431),
        S(43.139, 23.819),
        S(33.354, 23.806),
        S(25.546, 14.695),
        S(40.365, -14.354),
        S(-2.938, -8.947),
        S(11.058, 8.607),
        S(26.863, 26.448),
        S(39.159, 21.224),
        S(44.651, 19.960),
        S(37.601, 19.997),
        S(28.738, 11.085),
        S(11.997, -18.588),
        S(-41.745, -13.259),
        S(4.083, 3.349),
        S(33.063, 3.059),
        S(31.637, 16.108),
        S(40.942, 16.874),
        S(35.108, 0.978),
        S(31.912, -7.555),
        S(-30.281, -8.724),
        S(-44.130, -24.887),
        S(-23.413, -14.822),
        S(-4.146, -2.144),
        S(-2.875, 5.168),
        S(3.555, 5.613),
        S(13.192, -1.686),
        S(-4.690, -18.803),
        S(-23.700, -22.433),
        S(-73.599, -26.894),
        S(-56.346, -42.779),
        S(-49.956, -12.261),
        S(-27.125, -15.718),
        S(-30.896, -19.120),
        S(-13.930, -22.227),
        S(-49.403, -40.166),
        S(-71.656, -17.259),
    ],
    [
        S(6.776, -20.036),
        S(-63.172, 0.403),
        S(10.568, -18.725),
        S(-5.212, -3.364),
        S(-22.696, -5.014),
        S(3.790, -15.475),
        S(-50.176, -4.294),
        S(12.660, -23.896),
        S(-33.690, -13.383),
        S(-14.086, -0.250),
        S(-23.436, 2.095),
        S(-57.279, 4.434),
        S(-50.610, 5.447),
        S(-28.376, -0.268),
        S(-19.086, 1.404),
        S(-19.097, -22.450),
        S(-25.716, -1.138),
        S(0.187, 2.567),
        S(-7.045, 9.481),
        S(10.687, 1.270),
        S(2.864, 5.364),
        S(-2.510, 11.791),
        S(15.816, 0.351),
        S(-7.472, -2.260),
        S(-9.803, -4.138),
        S(3.848, 8.870),
        S(10.019, 12.280),
        S(8.790, 16.809),
        S(14.884, 15.921),
        S(12.330, 11.028),
        S(13.775, 7.104),
        S(-0.400, -9.936),
        S(3.625, -10.087),
        S(-3.972, 10.883),
        S(2.079, 17.275),
        S(26.849, 15.223),
        S(21.505, 13.757),
        S(9.945, 13.775),
        S(5.514, 4.419),
        S(16.549, -18.998),
        S(4.382, -8.519),
        S(18.159, 0.717),
        S(17.573, 12.918),
        S(9.503, 13.235),
        S(11.216, 18.112),
        S(12.415, 12.107),
        S(24.201, -3.268),
        S(8.446, -10.450),
        S(-9.449, -3.300),
        S(18.847, -11.553),
        S(14.131, -3.097),
        S(-8.956, 5.596),
        S(1.685, 4.572),
        S(26.499, -6.391),
        S(34.984, -12.513),
        S(1.450, -14.209),
        S(-15.279, -19.632),
        S(-17.719, -1.670),
        S(-14.716, -27.665),
        S(-29.114, -2.856),
        S(-14.922, -11.437),
        S(-16.950, -15.930),
        S(-7.998, -10.892),
        S(-20.965, -26.083),
    ],
    [
        S(12.100, 18.442),
        S(-9.058, 19.929),
        S(-36.588, 29.822),
        S(-35.398, 22.497),
        S(-27.148, 16.716),
        S(-27.153, 15.573),
        S(-4.921, 13.036),
        S(42.290, 5.674),
        S(-8.610, 17.285),
        S(-8.187, 19.315),
        S(-6.829, 20.708),
        S(-5.700, 11.822),
        S(-5.412, 5.951),
        S(-5.937, 5.667),
        S(-5.339, 6.444),
        S(28.275, 0.357),
        S(-12.149, 14.083),
        S(-9.148, 10.804),
        S(-6.721, 5.687),
        S(1.324, -3.603),
        S(8.002, -13.204),
        S(2.375, -9.134),
        S(19.297, -7.296),
        S(17.265, -6.334),
        S(-11.420, 10.827),
        S(-6.957, 4.080),
        S(3.821, 2.358),
        S(4.958, -4.131),
        S(13.832, -15.513),
        S(14.021, -15.257),
        S(18.643, -13.245),
        S(24.622, -12.611),
        S(-15.804, 6.547),
        S(-18.976, 4.898),
        S(-1.878, -1.121),
        S(12.774, -8.780),
        S(14.828, -13.293),
        S(1.388, -8.254),
        S(13.776, -14.006),
        S(8.261, -12.085),
        S(-27.917, 4.540),
        S(-8.881, -5.921),
        S(-3.131, -5.580),
        S(3.093, -9.139),
        S(11.786, -13.817),
        S(6.861, -16.433),
        S(33.083, -26.454),
        S(14.437, -20.818),
        S(-40.352, 3.431),
        S(-23.618, -2.191),
        S(-1.389, -7.816),
        S(1.533, -10.480),
        S(2.484, -14.810),
        S(3.396, -16.411),
        S(19.364, -24.762),
        S(-14.824, -11.305),
        S(2.918, -8.900),
        S(-9.963, 1.785),
        S(2.215, 3.390),
        S(16.917, -3.905),
        S(15.615, -10.734),
        S(10.720, -15.474),
        S(20.457, -16.703),
        S(9.862, -19.993),
    ],
    [
        S(-8.864, -0.708),
        S(-30.442, 14.186),
        S(-52.265, 34.819),
        S(-56.841, 32.481),
        S(-44.717, 27.400),
        S(-56.649, 27.543),
        S(-13.840, 3.229),
        S(6.246, -2.261),
        S(-10.225, -13.610),
        S(-20.970, 9.412),
        S(-32.454, 24.494),
        S(-73.107, 47.857),
        S(-83.859, 60.552),
        S(-43.481, 30.425),
        S(-27.160, 16.577),
        S(4.094, -2.945),
        S(-3.266, -18.848),
        S(4.828, -9.451),
        S(-15.274, 17.037),
        S(-7.857, 12.493),
        S(-11.550, 23.841),
        S(-6.678, 20.037),
        S(25.638, -5.158),
        S(12.689, -5.791),
        S(-14.065, -4.343),
        S(2.446, -1.372),
        S(-6.137, 9.850),
        S(-1.867, 16.986),
        S(5.398, 18.306),
        S(5.530, 18.904),
        S(17.747, 8.413),
        S(21.494, -7.534),
        S(7.678, -18.821),
        S(-5.936, 3.250),
        S(11.861, 0.150),
        S(19.612, 4.741),
        S(19.367, 8.814),
        S(19.216, 1.716),
        S(14.728, 3.460),
        S(21.661, -13.572),
        S(-4.020, -13.982),
        S(13.486, -12.927),
        S(23.330, -7.276),
        S(15.207, -0.509),
        S(23.891, -1.834),
        S(26.814, -5.435),
        S(39.694, -19.192),
        S(20.711, -21.284),
        S(-10.556, -16.312),
        S(7.610, -16.273),
        S(22.309, -25.574),
        S(17.456, -14.003),
        S(20.376, -15.480),
        S(32.143, -30.740),
        S(34.803, -37.303),
        S(19.570, -42.277),
        S(-0.703, -22.140),
        S(-29.314, -3.912),
        S(-9.489, -11.237),
        S(22.718, -65.747),
        S(4.067, -23.356),
        S(-12.161, -7.476),
        S(1.161, -21.099),
        S(-13.471, -23.349),
    ],
    [
        S(21.021, -34.495),
        S(43.975, -19.990),
        S(41.195, -14.982),
        S(-5.324, -2.807),
        S(6.179, -1.197),
        S(29.166, 1.723),
        S(43.169, -0.560),
        S(50.315, -30.460),
        S(6.119, -13.035),
        S(14.952, 17.723),
        S(-4.964, 17.279),
        S(10.699, 15.901),
        S(-21.165, 29.100),
        S(13.697, 32.812),
        S(38.941, 24.244),
        S(7.858, 6.818),
        S(-37.629, 0.064),
        S(12.781, 21.001),
        S(-26.716, 32.994),
        S(-48.714, 45.445),
        S(-29.787, 47.996),
        S(21.802, 41.856),
        S(25.200, 34.031),
        S(-0.032, 11.729),
        S(-47.283, 0.536),
        S(-35.809, 24.450),
        S(-57.023, 39.974),
        S(-104.835, 54.765),
        S(-88.421, 54.361),
        S(-71.794, 50.198),
        S(-55.937, 36.390),
        S(-74.601, 15.024),
        S(-53.990, -8.968),
        S(-53.845, 16.614),
        S(-66.092, 32.287),
        S(-99.684, 49.592),
        S(-102.446, 50.064),
        S(-71.841, 37.774),
        S(-67.569, 23.852),
        S(-91.024, 8.272),
        S(-22.751, -19.180),
        S(-2.924, 0.129),
        S(-40.998, 18.182),
        S(-59.199, 31.013),
        S(-58.154, 31.406),
        S(-49.673, 21.886),
        S(-21.368, 5.828),
        S(-28.811, -11.999),
        S(31.229, -36.788),
        S(13.527, -15.095),
        S(2.738, -2.311),
        S(-27.071, 5.850),
        S(-27.183, 8.911),
        S(-3.623, -1.307),
        S(22.738, -16.124),
        S(16.176, -32.767),
        S(-9.522, -63.763),
        S(60.049, -55.350),
        S(42.148, -40.647),
        S(-14.948, -34.202),
        S(44.845, -50.868),
        S(1.352, -38.423),
        S(45.284, -49.449),
        S(8.659, -68.853),
    ],
];
const THREAT: [[f32; 5]; 5] = [
    [-0.720, 1.005, 1.110, 0.408, 0.956],
    [0.170, -0.023, 0.801, 0.670, 0.684],
    [0.068, 0.652, -0.133, 0.649, 0.670],
    [0.262, 0.567, 0.595, 0.021, 1.029],
    [0.100, 0.422, 0.059, 0.094, 0.002],
];
const PROMO_BONUS: [f32; 2] = [1.088, -1.880];
const BAD_SEE_PENALTY: f32 = -2.615;
const CHECK_BONUS: f32 = 0.543;

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
        for pt in [PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King] {
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
