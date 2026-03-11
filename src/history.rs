use crate::{
    chess::{Board, Move},
    score::sigmoid_inv,
};

const BONUS_SCALE: f32 = 100.0;
pub const MAX_HISTORY: i16 = 8192;

pub struct History {
    entries: [[[HistoryEntry; 64]; 64]; 2],
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: [[[HistoryEntry(0); 64]; 64]; 2],
        }
    }

    pub fn get(&self, board: &Board, mv: Move) -> i32 {
        self.entries[board.stm() as usize][mv.from_sq().value() as usize]
            [mv.to_sq().value() as usize]
            .0 as i32
    }

    pub fn update(&mut self, board: &Board, mv: Move, score: f32) {
        self.entries[board.stm() as usize][mv.from_sq().value() as usize]
            [mv.to_sq().value() as usize]
            .update(score);
    }

    pub fn clear(&mut self) {
        self.entries
            .as_flattened_mut()
            .as_flattened_mut()
            .fill(HistoryEntry(0));
    }
}

#[derive(Clone, Copy)]
struct HistoryEntry(i16);

impl HistoryEntry {
    fn update(&mut self, score: f32) {
        let bonus = sigmoid_inv(score.clamp(0.001, 0.999), BONUS_SCALE) as i16;
        self.0 += bonus - bonus.abs() * self.0 / MAX_HISTORY;
    }
}
