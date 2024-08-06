mod types;

use types::{PieceType, Color, Piece};

fn main() {
    println!("{:?} {:?} {:?} {:?} {:?} {:?}", PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King);
    println!("{:?} {:?}", Color::White, Color::Black);
    let piece = Piece::new(Color::White, PieceType::King);
    println!("{} {:?} {:?}", piece, piece.color(), piece.piece_type());
}
