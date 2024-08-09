use crate::types::{Square, Color, PieceType, Bitboard, Piece};

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    stm: Color,
    castle_rights: u8,
    ep_square: Option<Square>,
}

impl Board {
    pub fn from_fen(fen: &str) -> Option<Board> {
        let board = Board::default();
        let mut iter = fen.chars();
        let mut curr = Square::A7;
        loop {
            let Some(c) = iter.next() else { return None };
            match c {
                '1'..='9' => {
                    curr += c as u8
                },
                'P' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Pawn))
                },
                'N' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Knight))
                },
                'B' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Bishop))
                },
                'R' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Rook))
                },
                'Q' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Queen))
                },
                'K' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::King))
                },
                'p' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Pawn))
                },
                'n' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Knight))
                },
                'b' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Bishop))
                },
                'r' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Rook))
                },
                'q' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::Queen))
                },
                'k' => {
                    board.addPiece(curr, Piece::new(Color::White, PieceType::King))
                },
                '/' => {
                    curr -= 16;
                },
                _ => break,
            };
        }

        if iter.next() == None {
            return None;
        };
        let Some(stm) = iter.next() else { return None };
        board.stm = if stm == 'w' {
            Color::White
        } else if stm == 'b' {
            Color::Black
        } else {
            return None;
        }

        

        board
    }

    fn default() -> Board {
        return Board {
            pieces: [Bitboard::EMPTY; 6],
            colors: [Bitboard::EMPTY; 2],
            stm: Color::White,
            castle_rights: 0,
            ep_square: None
        };
    }

    fn addPiece(&mut self, sq: Square, piece: Piece) {
        
    }
}
