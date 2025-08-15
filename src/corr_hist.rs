use crate::{
    chess::{Board, ZobristKey},
    types::Piece,
};

fn pawn_hash(board: &Board) -> ZobristKey {
    let mut key = ZobristKey::new();
    for piece in [Piece::WhitePawn, Piece::BlackPawn] {
        let mut pawns = board.colored_pieces(piece);
        while pawns.any() {
            key.toggle_piece(piece, pawns.poplsb());
        }
    }
    key
}

#[derive(Debug, Clone, Copy)]
struct CorrHistEntry {
    value: u16,
}

impl CorrHistEntry {
    const QUANT: f32 = 65535.0;

    fn get(&self) -> f32 {
        self.value as f32 / Self::QUANT
    }

    fn set(&mut self, val: f32) {
        self.value = (val * Self::QUANT) as u16
    }

    fn update(&mut self, val: f32, weight: f32) {
        self.set(self.get() * (1.0 - weight) + val * weight);
    }
}

pub struct CorrHist {
    entries: [[CorrHistEntry; 16384]; 2],
}

impl CorrHist {
    pub fn new() -> Self {
        Self {
            entries: [[CorrHistEntry { value: 0 }; 16384]; 2],
        }
    }
    pub fn get_corr(&self, board: &Board) -> f32 {
        let key = pawn_hash(board);
        self.entries[board.stm() as usize][(key.value() % 16384) as usize].get()
    }

    pub fn update_corr(&mut self, board: &Board, q: f32, static_eval: f32, visits: u32) {
        let key = pawn_hash(board);
        let entry = &mut self.entries[board.stm() as usize][(key.value() % 16384) as usize];
        let target = q - static_eval;
        let weight = ((visits as f32).powf(1.0 / 3.0) / 256.0).min(1.0 / 16.0);
        entry.update(target, weight);
    }
}
