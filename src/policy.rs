use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    eval::psqt_score,
    types::{Piece, PieceType, Square},
};

fn mopup_policy(board: &Board, mv: Move, bad_see: bool) -> f32 {
    if bad_see {
        return -3.0;
    }
	// let our_king_sq = board.king_sq(board.stm());
	let their_king_sq = board.king_sq(!board.stm());
	let dist_change = Square::chebyshev(mv.to_sq(), their_king_sq) - Square::chebyshev(mv.from_sq(), their_king_sq);
	return dist_change as f32 * 0.7;
}

pub fn get_policy(board: &Board, mv: Move) -> f32 {
    let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let pawn_protected = attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let captured_piece = board.piece_at(mv.to_sq());
    let cap_bonus = if let Some(captured) = captured_piece {
        match captured.piece_type() {
            PieceType::Pawn => 0.7,
            PieceType::Knight => 2.0,
            PieceType::Bishop => 2.0,
            PieceType::Rook => 3.0,
            PieceType::Queen => 4.5,
            _ => 0.0,
        }
    } else {
        0.0
    };
    let pawn_protected_penalty = if pawn_protected.has(mv.to_sq()) {
        match moving_piece.piece_type() {
            PieceType::Pawn => 0.6,
            PieceType::Knight => 1.9,
            PieceType::Bishop => 1.9,
            PieceType::Rook => 2.8,
            PieceType::Queen => 4.2,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let pawn_threat_evasion = if pawn_protected.has(mv.from_sq()) && !pawn_protected.has(mv.to_sq())
    {
        match moving_piece.piece_type() {
            PieceType::Pawn => 0.4,
            PieceType::Knight => 1.2,
            PieceType::Bishop => 1.2,
            PieceType::Rook => 2.4,
            PieceType::Queen => 3.5,
            _ => 0.0,
        }
    } else {
        0.0
    };

    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let psqt = if mv.kind() != MoveKind::Promotion {
        psqt_score(board, moving_piece.piece_type(), mv.to_sq(), board.stm())
            - psqt_score(board, moving_piece.piece_type(), mv.from_sq(), board.stm())
    } else {
        0
    };

    let promo_bonus = if mv.kind() == MoveKind::Promotion {
        match mv.promo_piece() {
            PieceType::Queen => 2.0,
            _ => -3.0,
        }
    } else {
        0.0
    };

    let bad_see_penalty = if pawn_protected_penalty > 0.0 {
        0.0
    } else if !see::see(board, mv, 0) {
        1.2
    } else {
        0.0
    };

    let check_bonus = if board.gives_direct_check(mv) {
        0.9
    } else {
        0.0
    };

    let base_policy = cap_bonus + promo_bonus + pawn_threat_evasion - bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt as f32 / 50.0;
		
	if board.colors(!board.stm()).one() {
		base_policy / 3.0 + check_bonus + mopup_policy(board, mv, bad_see_penalty > 0.0)
	} else {
		base_policy
	}
}
