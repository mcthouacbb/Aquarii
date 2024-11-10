use super::{attacks, Board, Move};
use crate::types::{Bitboard, Color, Piece, PieceType, Square};

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
    let eighth_rank = if board.stm() == Color::White {
        Bitboard::RANK_8
    } else {
        Bitboard::RANK_1
    };
    let third_rank = if board.stm() == Color::White {
        Bitboard::RANK_3
    } else {
        Bitboard::RANK_6
    };

    let push_offset = if board.stm() == Color::White { 8 } else { -8 };

    let pawns = board.colored_pieces(Piece::new(board.stm(), PieceType::Pawn));
    let pushes = attacks::pawn_pushes_bb(board.stm(), pawns) & !board.occ();
    let mut promo_pushes = pushes & eighth_rank;
    let mut non_promo_pushes = pushes ^ promo_pushes;

    while promo_pushes.any() {
        let sq = promo_pushes.poplsb();
        moves.push(Move::promo(sq - push_offset, sq, PieceType::Knight));
        moves.push(Move::promo(sq - push_offset, sq, PieceType::Bishop));
        moves.push(Move::promo(sq - push_offset, sq, PieceType::Rook));
        moves.push(Move::promo(sq - push_offset, sq, PieceType::Queen));
    }

    let mut double_pushes =
        attacks::pawn_pushes_bb(board.stm(), non_promo_pushes & third_rank) & !board.occ();

    while non_promo_pushes.any() {
        let sq = non_promo_pushes.poplsb();
        moves.push(Move::normal(sq - push_offset, sq))
    }

    while double_pushes.any() {
        let sq = double_pushes.poplsb();
        moves.push(Move::normal(sq - push_offset * 2, sq));
    }

    let mut west_caps =
        board.colors(!board.stm()) & attacks::pawn_west_attacks_bb(board.stm(), pawns);
    let mut promo_west_caps = west_caps & eighth_rank;
    west_caps ^= promo_west_caps;

    while west_caps.any() {
        let sq = west_caps.poplsb();
        moves.push(Move::normal(sq - push_offset + 1, sq));
    }

    while promo_west_caps.any() {
        let sq = promo_west_caps.poplsb();
        moves.push(Move::promo(sq - push_offset + 1, sq, PieceType::Knight));
        moves.push(Move::promo(sq - push_offset + 1, sq, PieceType::Bishop));
        moves.push(Move::promo(sq - push_offset + 1, sq, PieceType::Rook));
        moves.push(Move::promo(sq - push_offset + 1, sq, PieceType::Queen));
    }

    let mut east_caps =
        board.colors(!board.stm()) & attacks::pawn_east_attacks_bb(board.stm(), pawns);
    let mut promo_east_caps = east_caps & eighth_rank;
    east_caps ^= promo_east_caps;

    while east_caps.any() {
        let sq = east_caps.poplsb();
        moves.push(Move::normal(sq - push_offset - 1, sq));
    }

    while promo_east_caps.any() {
        let sq = promo_east_caps.poplsb();
        moves.push(Move::promo(sq - push_offset - 1, sq, PieceType::Knight));
        moves.push(Move::promo(sq - push_offset - 1, sq, PieceType::Bishop));
        moves.push(Move::promo(sq - push_offset - 1, sq, PieceType::Rook));
        moves.push(Move::promo(sq - push_offset - 1, sq, PieceType::Queen));
    }
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

    // if in check return

    if board.castling_rooks().color(board.stm()).king_side.is_some() {
        let king_dst = if board.stm() == Color::White {
            Square::G1
        } else {
            Square::G8
        };
        let rook_dst = if board.stm() == Color::White {
            Square::F1
        } else {
            Square::F8
        };

		let rook_sq = board.castling_rooks().color(board.stm()).king_side.unwrap();

        if (board.occ()
            & (attacks::line_between(sq, king_dst) | attacks::line_between(rook_sq, rook_dst)))
        .empty()
        {
            moves.push(Move::castle(sq, king_dst));
        }
    }

    if board.castling_rooks().color(board.stm()).queen_side.is_some() {
        let king_dst = if board.stm() == Color::White {
            Square::C1
        } else {
            Square::C8
        };
        let rook_dst = if board.stm() == Color::White {
            Square::D1
        } else {
            Square::D8
        };

		let rook_sq = board.castling_rooks().color(board.stm()).queen_side.unwrap();

        if (board.occ()
            & (attacks::line_between(sq, king_dst) | attacks::line_between(rook_sq, rook_dst)))
        .empty()
        {
            moves.push(Move::castle(sq, king_dst));
        }
    }
}
