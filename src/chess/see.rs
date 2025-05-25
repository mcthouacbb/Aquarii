use crate::types::{Bitboard, PieceType};

use super::{attacks, Board, Move, MoveKind};

fn see_piece_value(pt: PieceType) -> i32 {
    const VALUES: [i32; 6] = [100, 450, 450, 650, 1350, 0];
    VALUES[pt as usize]
}

fn pop_least_valuable(
    board: &Board,
    occupancy: &mut Bitboard,
    attackers: Bitboard,
) -> Option<PieceType> {
    for i in 0..6 {
        let pt = PieceType::from_raw(i);
        let pieces = attackers & board.pieces(pt);
        if pieces.any() {
            *occupancy ^= Bitboard::from_square(pieces.lsb());
            return Some(pt);
        }
    }
    None
}

// yoinked from stormphrax
pub fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    if mv.kind() != MoveKind::None {
        return true;
    }

    let mut score = if let Some(captured) = board.piece_at(mv.to_sq()) {
        see_piece_value(captured.piece_type())
    } else {
        0
    };

    score -= threshold;

    if score < 0 {
        return false;
    }

    let next = board.piece_at(mv.from_sq()).unwrap().piece_type();

    score -= see_piece_value(next);

    if score >= 0 {
        return true;
    }

    let square = mv.to_sq();

    let mut occupancy = board.occ();
    occupancy ^= Bitboard::from_square(square) ^ Bitboard::from_square(mv.from_sq());

    let mut attackers = board.all_attackers_to(square, occupancy);

    let mut us = !board.stm();

    loop {
        let our_attackers = attackers & board.colors(us);
        if our_attackers.empty() {
            break;
        }

        let next = pop_least_valuable(board, &mut occupancy, our_attackers);

        if next == Some(PieceType::Pawn)
            || next == Some(PieceType::Bishop)
            || next == Some(PieceType::Queen)
        {
            attackers |= attacks::bishop_attacks(square, occupancy)
                & (board.pieces(PieceType::Bishop) | board.pieces(PieceType::Queen));
        }

        if next == Some(PieceType::Rook) || next == Some(PieceType::Queen) {
            attackers |= attacks::rook_attacks(square, occupancy)
                & (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen));
        }

        attackers &= occupancy;
        score = -score
            - 1
            - if let Some(pt) = next {
                see_piece_value(pt)
            } else {
                0
            };

        us = !us;

        if score >= 0 {
            if next == Some(PieceType::King) && (attackers & board.colors(us)).any() {
                us = !us;
            }
            break;
        }
    }
    return board.stm() != us;
}
