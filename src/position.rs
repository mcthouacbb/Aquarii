use crate::chess::{Board, Move, ZobristKey};

#[derive(Clone)]
pub struct Position {
    board: Board,
    keys: Vec<ZobristKey>,
}

impl Position {
    pub fn new() -> Self {
        Self {
            board: Board::startpos(),
            keys: Vec::with_capacity(512),
        }
    }
    pub fn set_startpos(&mut self) {
        self.board = Board::startpos();
        self.keys.clear();
    }
    pub fn parse_fen(&mut self, fen: &str) -> bool {
        if let Some(board) = Board::from_fen(fen) {
            self.board = board;
            self.keys.clear();
            return true;
        }
        false
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn make_move(&mut self, mv: Move) {
        self.keys.push(self.board.zkey());
        self.board.make_move(mv);
    }

    pub fn is_drawn(&self) -> bool {
        self.board.is_drawn(&self.keys)
    }
}
