use super::{Board, Move, attacks};
use crate::types::{Bitboard, Piece, PieceType};

pub fn movegen(board: &Board) -> Vec<Move> {
	let mut result: Vec<Move> = Vec::new();

	gen_pawn_moves(board, &mut result);
	gen_knight_moves(board, &mut result);
	gen_bishop_moves(board, &mut result);
	gen_rook_moves(board, &mut result);
	gen_queen_moves(board, &mut result);
	gen_king_moves(board, &mut result);
	// replace true with board.is_legal(mv)
	result.retain(|&mv| true);

	result
}

fn gen_pawn_moves(board: &Board, moves: &mut Vec<Move>) {
	// let last_rank = if board.stm() == Color::White { Bitboard::RANK_8 } else { Bitboard::RANK_1 };

	// let pawns = board.colored_pieces(Piece::new(board.stm(), PieceType::Pawn));
	// let pushes = attacks::pawn_pushes_bb(color, bb);
	// let promo_pushes
}

fn gen_knight_moves(board: &Board, moves: &mut Vec<Move>) {
	let mut knights = board.colored_pieces(Piece::new(board.stm(), PieceType::Knight));
	while knights.any() {
		let sq = knights.poplsb();
		let mut attacks = attacks::knight_attacks(sq);
		attacks &= !board.colors(board.stm());
		while attacks.any() {
			moves.push(Move::normal(sq, attacks.poplsb()));
		}
	}
}

fn gen_bishop_moves(board: &Board, moves: &mut Vec<Move>) {
	let mut bishops = board.colored_pieces(Piece::new(board.stm(), PieceType::Bishop));
	while bishops.any() {
		let sq = bishops.poplsb();
		let mut attacks = attacks::bishop_attacks(sq, board.occ());
		attacks &= !board.colors(board.stm());
		while attacks.any() {
			moves.push(Move::normal(sq, attacks.poplsb()));
		}
	}
}

fn gen_rook_moves(board: &Board, moves: &mut Vec<Move>) {
	let mut rooks = board.colored_pieces(Piece::new(board.stm(), PieceType::Rook));
	while rooks.any() {
		let sq = rooks.poplsb();
		let mut attacks = attacks::rook_attacks(sq, board.occ());
		attacks &= !board.colors(board.stm());
		while attacks.any() {
			moves.push(Move::normal(sq, attacks.poplsb()));
		}
	}
}

fn gen_queen_moves(board: &Board, moves: &mut Vec<Move>) {
	let mut queens = board.colored_pieces(Piece::new(board.stm(), PieceType::Queen));
	while queens.any() {
		let sq = queens.poplsb();
		let mut attacks = attacks::queen_attacks(sq, board.occ());
		attacks &= !board.colors(board.stm());
		while attacks.any() {
			moves.push(Move::normal(sq, attacks.poplsb()));
		}
	}
}

fn gen_king_moves(board: &Board, moves: &mut Vec<Move>) {
	let sq = board.king_sq(board.stm());
	let mut attacks = attacks::king_attacks(sq);
	attacks &= !board.colors(board.stm());
	while attacks.any() {
		moves.push(Move::normal(sq, attacks.poplsb()));
	}
}
