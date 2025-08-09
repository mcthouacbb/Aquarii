use core::fmt;
use std::{
    num::NonZeroI16,
    ops::{Index, IndexMut, Range},
};

use arrayvec::ArrayVec;

use crate::{
    chess::{
        movegen::{self, MoveList},
        Board, Move,
    },
    policy,
};

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum GameResult {
    NonTerminal,
    Mated,
    Drawn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MateScore {
    Loss(u16),
    Win(u16),
}

fn sigmoid_inv(x: f32, scale: f32) -> f32 {
    scale * (x / (1.0 - x)).ln()
}

#[derive(Clone, Copy, PartialEq)]
pub enum Score {
    Win(u16),
    Draw,
    Loss(u16),
    Normal(f32),
}

impl Score {
    pub fn flip(&self) -> Self {
        match self {
            Self::Win(dist) => Self::Loss(*dist),
            Self::Draw => Self::Draw,
            Self::Loss(dist) => Self::Win(*dist),
            Self::Normal(score) => Self::Normal(1.0 - score),
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Win(dist) => write!(f, "win {} plies", *dist),
            Self::Draw => write!(f, "draw"),
            Self::Loss(dist) => write!(f, "loss {} plies", *dist),
            Self::Normal(score) => write!(f, "cp {}", sigmoid_inv(*score, 400.0).round()),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    first_child_idx: u32,
    child_count: u8,
    parent_move: Move,
    result: GameResult,
    mate_dist: Option<NonZeroI16>,
    policy: f32,
    wins: f32,
    visits: u32,
}

impl Node {
    fn new(mv: Move, policy: f32) -> Self {
        Node {
            first_child_idx: 0,
            child_count: 0,
            parent_move: mv,
            result: GameResult::NonTerminal,
            mate_dist: None,
            policy: policy,
            wins: 0.0,
            visits: 0,
        }
    }

    pub fn q(&self) -> f32 {
        self.wins / self.visits as f32
    }

    pub fn mate_score(&self) -> Option<MateScore> {
        if self.result == GameResult::Mated {
            Some(MateScore::Loss(0))
        } else if let Some(mate_dist) = self.mate_dist {
            let mate_dist = mate_dist.get() as i32;
            if mate_dist > 0 {
                Some(MateScore::Win(mate_dist as u16))
            } else {
                Some(MateScore::Loss(-mate_dist as u16))
            }
        } else {
            None
        }
    }

    pub fn score(&self) -> Score {
        if self.game_result() == GameResult::Drawn {
            Score::Draw
        } else if let Some(mate_score) = self.mate_score() {
            match mate_score {
                MateScore::Loss(dist) => Score::Loss(dist),
                MateScore::Win(dist) => Score::Win(dist),
            }
        } else {
            Score::Normal(self.q())
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.result != GameResult::NonTerminal
    }

    pub fn child_count(&self) -> u32 {
        self.child_count as u32
    }

    pub fn game_result(&self) -> GameResult {
        self.result
    }

    pub fn child_indices(&self) -> Range<u32> {
        self.first_child_idx..(self.first_child_idx + self.child_count as u32)
    }

    pub fn visits(&self) -> u32 {
        self.visits
    }

    pub fn parent_move(&self) -> Move {
        self.parent_move
    }

    pub fn policy(&self) -> f32 {
        self.policy
    }

    pub fn add_score(&mut self, score: f32) {
        self.visits += 1;
        self.wins += score;
    }

    pub fn set_mate_dist(&mut self, mate_dist: Option<NonZeroI16>) {
        self.mate_dist = mate_dist;
    }

    pub fn set_game_result(&mut self, result: GameResult) {
        self.result = result;
    }
}

fn softmax(vals: &mut ArrayVec<f32, 256>, max_val: f32) {
    let mut exp_sum = 0.0;
    for v in vals.iter_mut() {
        *v = (*v - max_val).exp();
        exp_sum += *v;
    }
    for v in vals.iter_mut() {
        *v /= exp_sum;
    }
}

pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn new(mb: u64) -> Self {
        let nodes = mb as usize * 1024 * 1024 / std::mem::size_of::<Node>();
        Self {
            nodes: Vec::with_capacity(nodes),
        }
    }

    fn build_tree_impl(&mut self, old_tree: &Tree, old_node_idx: u32, new_node_idx: u32) {
        let old_node = &old_tree[old_node_idx];
        let first_child_idx = self.nodes.len() as u32;
        if old_node.child_count == 0 {
            return;
        }

        {
            let new_node: &mut Node = &mut self[new_node_idx];
            new_node.child_count = old_node.child_count;
            new_node.first_child_idx = first_child_idx as u32;
        }

        for old_child_idx in old_node.child_indices() {
            let old_child = &old_tree[old_child_idx];
            self.nodes.push(old_child.clone());
        }

        for (iter, old_child_idx) in old_node.child_indices().enumerate() {
            let new_node = &self[new_node_idx];
            self.build_tree_impl(
                old_tree,
                old_child_idx,
                new_node.first_child_idx + iter as u32,
            );
        }
    }

    pub fn rebuild(old_tree: &Tree, node_idx: u32) -> Self {
        let mut new_tree = Self {
            nodes: Vec::with_capacity(old_tree.nodes.capacity()),
        };

        new_tree.nodes.push(old_tree[node_idx].clone());
        new_tree.build_tree_impl(old_tree, node_idx, 0);

        new_tree
    }

    pub fn add_root_node(&mut self) {
        assert!(self.nodes.len() == 0);
        self.nodes.push(Node::new(Move::NULL, 0.0));
    }

    pub fn expand_node(&mut self, node_idx: u32, board: &Board) {
        let mut moves = MoveList::new();
        movegen::movegen(board, &mut moves);

        // overflow check for later when implementing LRU
        // if self.nodes.len() + moves.len() > self.nodes.capacity() {
        //     return None
        // }

        let tmp = if node_idx == 0 { 3.0 } else { 1.0 };

        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;
        for mv in moves.iter() {
            let policy = policy::get_policy(board, *mv) / tmp;
            max_policy = max_policy.max(policy);
            policies.push(policy);
        }

        softmax(&mut policies, max_policy);

        let first_child_idx = self.nodes.len() as u32;
        let node = &mut self.nodes[node_idx as usize];
        node.first_child_idx = first_child_idx;
        node.child_count = moves.len() as u8;

        for (i, mv) in moves.iter().enumerate() {
            self.nodes.push(Node::new(*mv, policies[i]));
        }
    }

    pub fn relabel_policies(&mut self, node_idx: u32, board: &Board) {
        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;

        let tmp = if node_idx == 0 { 3.0 } else { 1.0 };

        for child_idx in self.nodes[node_idx as usize].child_indices() {
            let policy =
                policy::get_policy(board, self.nodes[child_idx as usize].parent_move) / tmp;
            max_policy = max_policy.max(policy);
            policies.push(policy);
        }

        softmax(&mut policies, max_policy);

        for (i, child_idx) in self.nodes[node_idx as usize].child_indices().enumerate() {
            self.nodes[child_idx as usize].policy = policies[i];
        }
    }

    pub fn size(&self) -> u32 {
        self.nodes.len() as u32
    }

    pub fn root_node(&self) -> u32 {
        0
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl Index<u32> for Tree {
    type Output = Node;
    fn index(&self, index: u32) -> &Self::Output {
        &self.nodes[index as usize]
    }
}

impl IndexMut<u32> for Tree {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.nodes[index as usize]
    }
}
