mod types;
mod chess;

use types::{Bitboard, PieceType, Color, Piece};
use chess::{Board};

fn main() {
    println!("{:?} {:?} {:?} {:?} {:?} {:?}", PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King);
    println!("{:?} {:?}", Color::White, Color::Black);
    let piece = Piece::new(Color::White, PieceType::King);
    println!("{} {:?} {:?}", piece, piece.color(), piece.piece_type());
    println!("{:?} {:?}", !Color::White, !Color::Black);

    let mut bb = Bitboard::FILE_C | Bitboard::FILE_E;
    let mut bb2 = !bb ^ Bitboard::RANK_3;

    println!("{}\n{}\n", bb, bb2);
    while bb.any() {
        let sq = bb.poplsb();
        println!("{:?}", sq);
    }

    while bb2.any() {
        let sq = bb2.poplsb();
        println!("{:?}", sq);
    }
}
