use super::{attacks, CastlingRooks, Move, MoveKind};
use crate::types::{Bitboard, Color, Piece, PieceType, Square};
use std::fmt;

#[derive(Clone)]
pub struct Board {
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
    checkers: Bitboard,
    diag_pinned: Bitboard,
    hv_pinned: Bitboard,
    castling_rooks: CastlingRooks,
    stm: Color,
    ep_square: Option<Square>,
    half_move_clock: u8,
}

impl Board {
    const STARTPOS_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut board = Self::empty();

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
                }
                'P' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::Pawn),
                ),
                'N' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::Knight),
                ),
                'B' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::Bishop),
                ),
                'R' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::Rook),
                ),
                'Q' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::Queen),
                ),
                'K' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::White, PieceType::King),
                ),
                'p' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::Pawn),
                ),
                'n' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::Knight),
                ),
                'b' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::Bishop),
                ),
                'r' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::Rook),
                ),
                'q' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::Queen),
                ),
                'k' => board.add_piece(
                    Square::from_raw(curr as u8),
                    Piece::new(Color::Black, PieceType::King),
                ),
                '/' => {
                    if curr != 64 - rows * 8 {
                        return None;
                    }
                    rows += 1;
                    curr -= 16;
                    // cancel out extra += 1 at the end
                    curr -= 1;
                }
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
                    board.castling_rooks.color_mut(Color::White).king_side = Some(Square::H1);
                }
                'Q' => {
                    board.castling_rooks.color_mut(Color::White).queen_side = Some(Square::A1);
                }
                'k' => {
                    board.castling_rooks.color_mut(Color::Black).king_side = Some(Square::H8);
                }
                'q' => {
                    board.castling_rooks.color_mut(Color::Black).queen_side = Some(Square::A8);
                }
                '-' => {
                    if parts[2].len() != 1 {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            }
        }

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
            board.ep_square = Some(Square::from_rank_file(
                rank as u8 - '1' as u8,
                file as u8 - 'a' as u8,
            ));
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

        board.update_check_info();

        Some(board)
    }

    pub fn startpos() -> Self {
        Self::from_fen(Self::STARTPOS_FEN).unwrap()
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut last_file = -1;
            for file in 0..8 {
                let sq = Square::from_rank_file(rank, file);
                match self.piece_at(sq) {
                    Some(piece) => {
                        let diff = sq as i32 - rank as i32 * 8 - last_file - 1;
                        if diff > 0 {
                            fen.push(std::char::from_digit(diff as u32, 10).unwrap());
                        }
                        fen.push(piece.char_repr());
                        last_file = sq as i32 - rank as i32 * 8;
                    }
                    None => {}
                }
            }
            let diff: i32 = 7 - last_file;
            if diff > 0 {
                fen.push(std::char::from_digit(diff as u32, 10).unwrap());
            }
            if rank != 0 {
                fen.push('/');
            }
        }

        fen += if self.stm == Color::White {
            " w "
        } else {
            " b "
        };
        fen += format!("{}", self.castling_rooks).as_str();
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

    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let from_pce = self.piece_at(from).unwrap();
        self.ep_square = None;
        match mv.kind() {
            MoveKind::None => {
                if let Some(captured) = self.piece_at(to) {
                    self.remove_piece(to, captured);
                }

                self.remove_piece(from, from_pce);
                self.add_piece(to, from_pce);

                if from_pce.piece_type() == PieceType::Pawn {
                    self.half_move_clock = 0;
                    if (from - to).abs() == 16 {
                        self.ep_square =
                            Some(Square::from_raw(((from as i32 + to as i32) / 2) as u8))
                    }
                }
            }
            MoveKind::Promotion => {
                if let Some(captured) = self.piece_at(to) {
                    self.remove_piece(to, captured);
                }
                self.remove_piece(from, from_pce);
                self.add_piece(to, Piece::new(self.stm, mv.promo_piece()))
            }
            MoveKind::Enpassant => {
                self.remove_piece(from, from_pce);
                self.add_piece(to, from_pce);

                let cap_sq = if self.stm == Color::White {
                    to - 8
                } else {
                    to + 8
                };
                self.remove_piece(cap_sq, self.piece_at(cap_sq).unwrap());
            }
            MoveKind::Castle => {
                // from = king_sq, to = rook_sq
                let rook = self.piece_at(to).unwrap();
                if to > from {
                    // king side
                    let king_to = if self.stm == Color::White {
                        Square::G1
                    } else {
                        Square::G8
                    };
                    let rook_to = if self.stm == Color::White {
                        Square::F1
                    } else {
                        Square::F8
                    };

                    self.remove_piece(from, from_pce);
                    self.remove_piece(to, rook);
                    self.add_piece(king_to, from_pce);
                    self.add_piece(rook_to, rook);
                } else {
                    // queen side
                    let king_to = if self.stm == Color::White {
                        Square::C1
                    } else {
                        Square::C8
                    };
                    let rook_to = if self.stm == Color::White {
                        Square::D1
                    } else {
                        Square::D8
                    };

                    self.remove_piece(from, from_pce);
                    self.remove_piece(to, rook);
                    self.add_piece(king_to, from_pce);
                    self.add_piece(rook_to, rook);
                }
            }
        }

        self.stm = !self.stm;

        self.update_check_info();
    }

    pub fn stm(&self) -> Color {
        self.stm
    }

    pub fn colors(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    pub fn occ(&self) -> Bitboard {
        self.colors(Color::White) | self.colors(Color::Black)
    }

    pub fn pieces(&self, piece: PieceType) -> Bitboard {
        self.pieces[piece as usize]
    }

    pub fn colored_pieces(&self, piece: Piece) -> Bitboard {
        self.colors(piece.color()) & self.pieces(piece.piece_type())
    }

    pub fn king_sq(&self, color: Color) -> Square {
        self.colored_pieces(Piece::new(color, PieceType::King))
            .lsb()
    }

    pub fn castling_rooks(&self) -> CastlingRooks {
        self.castling_rooks
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

    pub fn attackers_to(&self, sq: Square) -> Bitboard {
        let diags = self.pieces(PieceType::Bishop) | self.pieces(PieceType::Queen);
        let hvs = self.pieces(PieceType::Rook) | self.pieces(PieceType::Queen);
        let wpawns = self.colored_pieces(Piece::new(Color::Black, PieceType::Pawn));
        let bpawns = self.colored_pieces(Piece::new(Color::White, PieceType::Pawn));
        (attacks::king_attacks(sq) & self.pieces(PieceType::King))
            | (attacks::knight_attacks(sq) & self.pieces(PieceType::Knight))
            | (attacks::bishop_attacks(sq, self.occ() ^ self.pieces(PieceType::King)) & diags)
            | (attacks::rook_attacks(sq, self.occ() ^ self.pieces(PieceType::King)) & hvs)
            | (attacks::pawn_attacks(Color::White, sq) & wpawns)
            | (attacks::pawn_attacks(Color::Black, sq) & bpawns)
    }

    pub fn colored_attackers_to(&self, sq: Square, c: Color) -> Bitboard {
        self.attackers_to(sq) & self.colors(c)
    }

    pub fn checkers(&self) -> Bitboard {
        self.checkers
    }

    pub fn pinned(&self) -> Bitboard {
        self.hv_pinned | self.diag_pinned
    }

    pub fn diag_pinned(&self) -> Bitboard {
        self.diag_pinned
    }

    pub fn hv_pinned(&self) -> Bitboard {
        self.hv_pinned
    }

    pub fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    fn empty() -> Board {
        Self {
            pieces: [Bitboard::NONE; 6],
            colors: [Bitboard::NONE; 2],
            checkers: Bitboard::NONE,
            diag_pinned: Bitboard::NONE,
            hv_pinned: Bitboard::NONE,
            castling_rooks: CastlingRooks::DEFAULT,
            stm: Color::White,
            ep_square: None,
            half_move_clock: 0,
        }
    }

    fn add_piece(&mut self, sq: Square, piece: Piece) {
        let sq_bb = Bitboard::from_square(sq);
        self.pieces[piece.piece_type() as usize] |= sq_bb;
        self.colors[piece.color() as usize] |= sq_bb;
    }

    fn remove_piece(&mut self, sq: Square, piece: Piece) {
        assert!(self.piece_at(sq).unwrap() == piece);
        let sq_bb = Bitboard::from_square(sq);
        self.pieces[piece.piece_type() as usize] ^= sq_bb;
        self.colors[piece.color() as usize] ^= sq_bb;
    }

    fn update_check_info(&mut self) {
        let king_sq = self.king_sq(self.stm());
        self.checkers = self.colored_attackers_to(king_sq, !self.stm());

        // this includes enemy pieces as pinned but they are ignored so it is fine
        self.diag_pinned = Bitboard::NONE;
        self.hv_pinned = Bitboard::NONE;

        let queens = self.colored_pieces(Piece::new(!self.stm(), PieceType::Queen));
        let rooks = self.colored_pieces(Piece::new(!self.stm(), PieceType::Rook));
        let bishops = self.colored_pieces(Piece::new(!self.stm(), PieceType::Bishop));

        let mut diag_attackers =
            attacks::bishop_attacks(king_sq, Bitboard::NONE) & (bishops | queens);

        let block_mask = self.occ() ^ diag_attackers;

        while diag_attackers.any() {
            let attacker = diag_attackers.poplsb();

            let between = attacks::line_between(king_sq, attacker) & block_mask;
            if between.one() {
                self.diag_pinned |= between;
            }
        }

        let mut hv_attackers = attacks::rook_attacks(king_sq, Bitboard::NONE) & (rooks | queens);

        let block_mask = self.occ() ^ hv_attackers;

        while hv_attackers.any() {
            let attacker = hv_attackers.poplsb();

            let between = attacks::line_between(king_sq, attacker) & block_mask;
            if between.one() {
                self.diag_pinned |= between;
            }
        }
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
                    }
                    None => {
                        write!(f, ".")?;
                    }
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "stm: {}", self.stm)?;
        writeln!(f, "castling rights: {}", self.castling_rooks)?;
        match self.ep_square {
            Some(sq) => {
                writeln!(f, "ep square: {}", sq)?;
            }
            None => {
                writeln!(f, "ep square: N/A")?;
            }
        }
        writeln!(f, "half move clock: {}", self.half_move_clock)?;
        writeln!(f, "fen: {}", self.to_fen())?;
        Ok(())
    }
}
