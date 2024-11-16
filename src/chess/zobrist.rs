use crate::types::{Piece, Square};

use super::CastlingRooks;

// lol
const fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

struct ZobristKeys {
    piece_squares: [[u64; 12]; 64],
    castling_rights: [u64; 16],
    enpassant: [u64; 8],
    stm: u64,
}

const ZOBRIST_KEYS: ZobristKeys = {
    let mut result = ZobristKeys {
        piece_squares: [[0; 12]; 64],
        castling_rights: [0; 16],
        enpassant: [0; 8],
        stm: 0,
    };

    let mut rand = 0x3519A84F;
    rand = xorshift64(rand);

    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < 12 {
            result.piece_squares[i][j] = rand;
            rand = xorshift64(rand);
            j += 1;
        }
        i += 1;
    }

    let mut i = 0;
    while i < 16 {
        result.castling_rights[i] = rand;
        rand = xorshift64(rand);
        i += 1;
    }

    let mut i = 0;
    while i < 8 {
        result.enpassant[i] = rand;
        rand = xorshift64(rand);
        i += 1;
    }

    result.stm = rand;

    result
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZobristKey(u64);

impl ZobristKey {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn toggle_stm(&mut self) {
        self.0 ^= ZOBRIST_KEYS.stm;
    }

    pub fn toggle_piece(&mut self, piece: Piece, square: Square) {
        self.0 ^= ZOBRIST_KEYS.piece_squares[square as usize][piece as usize];
    }

    pub fn toggle_castle_rights(&mut self, castle_rooks: CastlingRooks) {
        self.0 ^= ZOBRIST_KEYS.castling_rights[castle_rooks.right_bits() as usize];
    }

    pub fn toggle_ep_square(&mut self, ep_square: Square) {
        self.0 ^= ZOBRIST_KEYS.enpassant[ep_square.file() as usize];
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
