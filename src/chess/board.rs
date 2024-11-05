use crate::types::{Square, Color, PieceType, Bitboard, Piece};
use std::fmt;

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    stm: Color,
    castle_rights: u8,
    ep_square: Option<Square>,
}

impl Board {
    const STARTPOS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn from_fen(fen: &str) -> Option<Board> {
        let mut board = Board::empty();
        let mut iter = fen.chars();
        let mut curr = Square::A8 as i32;
        loop {
            let Some(c) = iter.next() else { return None };
            match c {
                '1'..='9' => {
                    curr += c as i32 - '0' as i32;
                    // cancel out extra += 1 at the end
                    curr -= 1;
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
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::Pawn))
                },
                'n' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::Knight))
                },
                'b' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::Bishop))
                },
                'r' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::Rook))
                },
                'q' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::Queen))
                },
                'k' => {
                    board.add_piece(Square::from(curr as u8), Piece::new(Color::Black, PieceType::King))
                },
                '/' => {
                    curr -= 16;
                    // cancel out extra += 1 at the end
                    curr -= 1;
                },
                _ => break,
            };
            curr += 1;
        }

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
        Board::from_fen(Self::STARTPOS_FEN).unwrap()
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        let c = if self.colors[Color::White as usize].has(sq) {
           Color::White
        } else if self.colors[Color::Black as usize].has(sq) {
            Color::Black
        } else {
            return None;
        };

        if self.pieces[PieceType::Pawn as usize].has(sq) {
            Some(Piece::new(c, PieceType::Pawn))
        } else if self.pieces[PieceType::Knight as usize].has(sq) {
            Some(Piece::new(c, PieceType::Knight))
        } else if self.pieces[PieceType::Bishop as usize].has(sq) {
            Some(Piece::new(c, PieceType::Bishop))
        } else if self.pieces[PieceType::Rook as usize].has(sq) {
            Some(Piece::new(c, PieceType::Rook))
        } else if self.pieces[PieceType::Queen as usize].has(sq) {
            Some(Piece::new(c, PieceType::Queen))
        } else if self.pieces[PieceType::King as usize].has(sq) {
            Some(Piece::new(c, PieceType::King))
        } else {
            unreachable!();
        }
    }

    fn empty() -> Board {
        Board {
            pieces: [Bitboard::EMPTY; 6],
            colors: [Bitboard::EMPTY; 2],
            stm: Color::White,
            castle_rights: 0,
            ep_square: None
        }
    }

    fn add_piece(&mut self, sq: Square, piece: Piece) {
        let sq_bb = Bitboard::from_square(sq);
        self.pieces[piece.piece_type() as usize] |= sq_bb;
        self.colors[piece.color() as usize] |= sq_bb;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::from_rank_file(rank, file);
                let p = self.piece_at(sq);
                match p {
                    Some(piece) => {
                        let c = match piece {
                            Piece::WhitePawn => 'P',
                            Piece::BlackPawn => 'p',
                            Piece::WhiteKnight => 'N',
                            Piece::BlackKnight => 'n',
                            Piece::WhiteBishop => 'B',
                            Piece::BlackBishop => 'b',
                            Piece::WhiteRook => 'R',
                            Piece::BlackRook => 'r',
                            Piece::WhiteQueen => 'Q',
                            Piece::BlackQueen => 'q',
                            Piece::WhiteKing => 'K',
                            Piece::BlackKing => 'k',
                        };
                        write!(f, "{}", c)?;
                    },
                    None => {
                        write!(f, ".")?;
                    }
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
