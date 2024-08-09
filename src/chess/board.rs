use crate::types::{Square, Color, PieceType, Bitboard, Piece};

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    stm: Color,
    castle_rights: u8,
    ep_square: Option<Square>,
}

impl Board {
    const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn from_fen(fen: &str) -> Option<Board> {
        let mut board = Board::empty();
        let mut iter = fen.chars();
        let mut curr = Square::A7 as i32;
        loop {
            let Some(c) = iter.next() else { return None };
            match c {
                '1'..='9' => {
                    curr += c as i32 - '0' as i32;
                },
                'P' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Pawn))
                },
                'N' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Knight))
                },
                'B' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Bishop))
                },
                'R' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Rook))
                },
                'Q' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Queen))
                },
                'K' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::King))
                },
                'p' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Pawn))
                },
                'n' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Knight))
                },
                'b' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Bishop))
                },
                'r' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Rook))
                },
                'q' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::Queen))
                },
                'k' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::White, PieceType::King))
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
        };

        // !TODO
        // castling rights, ep square, half move clock and full move clock

        Some(board)
    }

    pub fn startpos() -> Board {
        return Board::from_fen(Self::STARTPOS_FEN).unwrap();
    }

    fn empty() -> Board {
        return Board {
            pieces: [Bitboard::EMPTY; 6],
            colors: [Bitboard::EMPTY; 2],
            stm: Color::White,
            castle_rights: 0,
            ep_square: None
        };
    }

    fn add_piece(&mut self, sq: Square, piece: Piece) {
        let sq_bb = Bitboard::from_square(sq);
        self.pieces[piece.piece_type() as usize] |= sq_bb;
        self.colors[piece.color() as usize] |= sq_bb;
    }
}
