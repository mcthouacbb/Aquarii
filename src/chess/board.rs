use crate::types::{Square, Color, PieceType, Bitboard, Piece};
use super::castle_rights::CastleRights;
use std::fmt;

pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    stm: Color,
    castle_rights: CastleRights,
    ep_square: Option<Square>,
    half_move_clock: u8
}

impl Board {
    const STARTPOS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn from_fen(fen: &str) -> Option<Board> {
        let mut board = Board::empty();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return None;
        }

        let mut curr = Square::A8 as i32;
        let mut rows = 0;
        for c in parts[0].chars() {
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
                    if curr != 64 - rows * 8 {
                        return None;
                    }
                    rows += 1;
                    curr -= 16;
                    // cancel out extra += 1 at the end
                    curr -= 1;
                },
                _ => return None,
            };
            curr += 1;
        }

        if curr != 8 || rows != 7 {
            return None;
        }

        if parts[1].len() != 1 {
            return None;
        }

        let stm = parts[1].chars().next().unwrap();
        board.stm = if stm == 'w' {
            Color::White
        } else if stm == 'b' {
            Color::Black
        } else {
            return None;
        };

        if parts[2].len() == 0 || parts[2].len() > 4 {
            return None;
        }

        for c in parts[2].chars() {
            match c {
                'K' => {
                    board.castle_rights |= CastleRights::WHITE_KING_SIDE;
                },
                'Q' => {
                    board.castle_rights |= CastleRights::WHITE_QUEEN_SIDE;
                },
                'k' => {
                    board.castle_rights |= CastleRights::BLACK_KING_SIDE;
                },
                'q' => {
                    board.castle_rights |= CastleRights::BLACK_QUEEN_SIDE;
                },
                '-' => {
                    if parts[2].len() != 1 {
                        return None;
                    }
                },
                _ => {
                    return None;
                }
            }
        }


        // !TODO
        // ep square, half move clock and full move clock

        if parts[3].len() == 0 || parts[3].len() > 2 {
            return None;
        }

        if parts[3].len() == 1 && parts[3].chars().next().unwrap() != '-' {
            return None;
        }

        if parts[3].len() == 2 {
            // trash
            let mut iter = parts[3].chars();
            let file = iter.next().unwrap();
            let rank = iter.next().unwrap();
            board.ep_square = Some(Square::from_rank_file(rank as u8 - '1' as u8, file as u8 - 'a' as u8));
        }

        match parts[4].parse::<u8>() {
            Ok(n) => {
                board.half_move_clock = n;
            }
            Err(_) => {
                return None;
            }
        }
        if board.half_move_clock > 100 {
            return None;
        }

        Some(board)
    }

    pub fn startpos() -> Board {
        Board::from_fen(Self::STARTPOS_FEN).unwrap()
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut lastFile = -1;
            for file in 0..8 {
                let sq = Square::from_rank_file(rank, file);
                match self.piece_at(sq) {
                    Some(piece) => {
                        let diff = sq as i32 - rank as i32 * 8 - lastFile - 1;
                        if diff > 0 {
                            fen.push(std::char::from_digit(diff as u32, 10).unwrap());
                        }
                        fen.push(piece.char_repr());
                        lastFile = sq as i32 - rank as i32 * 8;
                    }
                    None => {

                    }
                }
            }
            let diff: i32 = 7 - lastFile;
            if diff > 0 {
                fen.push(std::char::from_digit(diff as u32, 10).unwrap());
            }
            if rank != 0 {
                fen.push('/');
            }
        }

        fen += if self.stm == Color::White { " w " } else { " b " };
        fen += format!("{}", self.castle_rights).as_str();
        match self.ep_square {
            Some(sq) => {
                fen += format!(" {} ", sq).to_lowercase().as_str();
            }
            None => {
                fen += " - ";
            }
        }

        fen += format!("{} 1", self.half_move_clock).as_str();

        fen
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
            castle_rights: CastleRights::NONE,
            ep_square: None,
            half_move_clock: 0
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
                        write!(f, "{}", piece.char_repr())?;
                    },
                    None => {
                        write!(f, ".")?;
                    }
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "stm: {}", self.stm)?;
        writeln!(f, "castling rights: {}", self.castle_rights)?;
        match self.ep_square {
            Some(sq) => {
                writeln!(f, "ep square: {}", sq)?;
            },
            None => {
                writeln!(f, "ep square: N/A")?;
            }
        }
        writeln!(f, "half move clock: {}", self.half_move_clock)?;
        writeln!(f, "fen: {}", self.to_fen())?;
        Ok(())
    }
}
