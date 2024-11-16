use std::{ops::Range, time::Instant};

use crate::{
    chess::{
        movegen::{movegen, MoveList},
        Board, Move,
    },
    types::{Color, Piece, PieceType},
};

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum GameResult {
    NonTerminal,
    Mated,
    Drawn,
}

struct Node {
    first_child_idx: u32,
    child_count: u8,
    parent_move: Move,
    result: GameResult,
    wins: f32,
    visits: u32,
}

impl Node {
    fn new(mv: Move) -> Self {
        Node {
            first_child_idx: 0,
            child_count: 0,
            parent_move: mv,
            result: GameResult::NonTerminal,
            wins: 0.0,
            visits: 0,
        }
    }

    fn q(&self) -> f32 {
        self.wins / self.visits as f32
    }

    fn is_terminal(&self) -> bool {
        self.result != GameResult::NonTerminal
    }

    fn child_indices(&self) -> Range<u32> {
        self.first_child_idx..(self.first_child_idx + self.child_count as u32)
    }
}

fn sigmoid(x: f32, scale: f32) -> f32 {
    1.0 / (1.0 + (-x / scale).exp())
}

fn sigmoid_inv(x: f32, scale: f32) -> f32 {
    scale * (x / (1.0 - x)).ln()
}

pub struct MCTS {
    nodes: Vec<Node>,
    iters: u32,
    root_board: Board,
    board: Board,
    selection: Vec<u32>,
}

impl MCTS {
    const CUCT: f32 = std::f32::consts::SQRT_2;
    const EVAL_SCALE: f32 = 200.0;

    pub fn new(max_nodes: u32) -> Self {
        Self {
            nodes: Vec::with_capacity(max_nodes as usize),
            iters: 0,
            root_board: Board::startpos(),
            board: Board::startpos(),
            selection: Vec::new(),
        }
    }

    pub fn select_leaf(&mut self) {
        self.selection.clear();
        let mut node_idx = 0u32;
        self.selection.push(node_idx);

        loop {
            let node = &self.nodes[node_idx as usize];
            if node.is_terminal() || node.child_count == 0 {
                break;
            } else {
                let mut best_uct = -1f32;
                let mut best_child_idx = 0u32;
                for child_idx in node.child_indices() {
                    let child = &self.nodes[child_idx as usize];
                    let q = if child.visits == 0 {
                        // TODO: inf root fpu
                        0.5
                    } else {
                        // 1 - child q because child q is from opposite perspective of current node
                        1.0 - child.q()
                    };
                    let expl = ((node.visits as f32).ln() / (child.visits + 1) as f32).sqrt();
                    let uct = q + Self::CUCT * expl;

                    if uct > best_uct {
                        best_child_idx = child_idx;
                        best_uct = uct;
                    }
                }

                node_idx = best_child_idx;
                let child = &self.nodes[best_child_idx as usize];
                self.board.make_move(child.parent_move);
                self.selection.push(node_idx);
            }
        }
    }

    pub fn expand_node(&mut self, node_idx: u32) {
        let mut moves = MoveList::new();
        movegen(&self.board, &mut moves);

        if moves.len() == 0 {
            let node = &mut self.nodes[node_idx as usize];
            node.result = if self.board.checkers().any() {
                GameResult::Mated
            } else {
                GameResult::Drawn
            };
        } else {
            let first_child_idx = self.nodes.len() as u32;
            let node = &mut self.nodes[node_idx as usize];
            node.first_child_idx = first_child_idx;
            node.child_count = moves.len() as u8;
            for mv in moves.iter() {
                self.nodes.push(Node::new(*mv));
            }
        }
    }

    pub fn eval_wdl(&self) -> f32 {
        let stm = self.board.stm();
        let material: i32 = 100
            * (self.board.piece_count(stm, PieceType::Pawn)
                - self.board.piece_count(!stm, PieceType::Pawn))
            + 300
                * (self.board.piece_count(stm, PieceType::Knight)
                    - self.board.piece_count(!stm, PieceType::Knight))
            + 300
                * (self.board.piece_count(stm, PieceType::Bishop)
                    - self.board.piece_count(!stm, PieceType::Bishop))
            + 500
                * (self.board.piece_count(stm, PieceType::Rook)
                    - self.board.piece_count(!stm, PieceType::Rook))
            + 900
                * (self.board.piece_count(stm, PieceType::Queen)
                    - self.board.piece_count(!stm, PieceType::Queen));

        sigmoid(material as f32, Self::EVAL_SCALE)
    }

    pub fn simulate(&self) -> f32 {
        let leaf = &self.nodes[*self.selection.last().unwrap() as usize];
        match leaf.result {
            GameResult::Drawn => 0.5,
            GameResult::Mated => 0.0,
            GameResult::NonTerminal => self.eval_wdl(),
        }
    }

    pub fn backprop(&mut self, mut result: f32) {
        for node_idx in self.selection.iter().rev() {
            let node = &mut self.nodes[*node_idx as usize];

            node.visits += 1;
            node.wins += result;

            result = 1.0 - result;
        }
    }

    pub fn perform_one_iter(&mut self) {
        self.board = self.root_board.clone();
        self.select_leaf();

        let leaf_idx = *self.selection.last().unwrap();
        let leaf = &self.nodes[leaf_idx as usize];
        if leaf.child_count == 0 {
            self.expand_node(leaf_idx);
        }

        let result = self.simulate();

        self.backprop(result);
    }

    pub fn get_best_move(&self) -> Move {
        let root_node = &self.nodes[0];
        let mut best_visits = 0;
        let mut best_move = Move::NULL;
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            if child_node.visits > best_visits {
                best_visits = child_node.visits;
                best_move = child_node.parent_move;
            }
        }
        best_move
    }

    pub fn display_tree(&self) {
        let root_node = &self.nodes[0];
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            println!(
                "{} => {} visits {} cp",
                child_node.parent_move,
                child_node.visits,
                sigmoid_inv(1.0 - child_node.q(), Self::EVAL_SCALE)
            );
        }
    }

    pub fn run(&mut self, iters: u32, report: bool, board: &Board) -> Move {
        self.root_board = board.clone();
        self.board = self.root_board.clone();

        self.nodes.clear();
        self.iters = 0;
        self.nodes.push(Node::new(Move::NULL));
        self.expand_node(0);
        self.nodes[0].visits += 1;
        self.nodes[0].wins += self.eval_wdl();

        let mut total_depth = 0;
        let mut prev_depth = 0;

        let mut nodes = 0;

        let start_time = Instant::now();

        while self.iters < iters {
            self.perform_one_iter();

            total_depth += (self.selection.len() - 1) as u32;
            self.iters += 1;

            nodes += self.selection.len();

            let curr_depth = total_depth / self.iters;
            if curr_depth > prev_depth {
                prev_depth = curr_depth;
                if report {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    println!(
                        "info depth {} nodes {} time {} nps {} score cp {} pv {}",
                        curr_depth,
                        nodes,
                        elapsed,
                        (nodes as f64 / elapsed as f64) as u64,
                        sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE),
                        self.get_best_move()
                    );
                }
            }
        }

        let curr_depth = total_depth / self.iters;
        let elapsed = start_time.elapsed().as_secs_f64();
        println!(
            "info depth {} nodes {} time {} nps {} score cp {} pv {}",
            curr_depth,
            nodes,
            elapsed,
            (nodes as f64 / elapsed as f64) as u64,
            sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE),
            self.get_best_move()
        );

        self.get_best_move()
    }
}
