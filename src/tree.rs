use core::fmt;
use std::{
    num::NonZeroI16, ops::{Index, IndexMut}
};

use arrayvec::ArrayVec;

use crate::{
    chess::{
        movegen::{self, MoveList}, Board, Move
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeIndex(u32);

impl NodeIndex {
    pub const INDEX_BITS: u32 = !(1u32 << 31);
    pub const NULL: Self = Self(u32::MAX);

    pub fn new(half: u8, index: u32) -> Self {
        Self(((half as u32) << 31) | index)
    }

    pub fn half(&self) -> u8 {
        (self.0 >> 31) as u8
    }

    pub fn index(&self) -> u32 {
        self.0 & Self::INDEX_BITS
    }
}

impl std::ops::Add<u32> for NodeIndex {
    type Output = NodeIndex;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u32> for NodeIndex {
    fn add_assign(&mut self, rhs: u32) {
        *self = *self + rhs;
    }
}

pub struct NodeIndexIter {
    start: NodeIndex,
    end: NodeIndex,
    curr: NodeIndex,
}

impl NodeIndexIter {
    fn new(start: NodeIndex, end: NodeIndex) -> Self {
        Self {
            start: start,
            end: end,
            curr: start
        }
    }
}

impl Iterator for NodeIndexIter {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;
        self.curr += 1;

        if curr == self.end {
            None
        } else {
            Some(curr)
        }
    }
}

#[derive(Clone)]
pub struct Node {
    first_child_idx: NodeIndex,
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
            first_child_idx: NodeIndex::NULL,
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

    pub fn child_indices(&self) -> NodeIndexIter {
        NodeIndexIter::new(self.first_child_idx, self.first_child_idx + self.child_count())
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

pub struct Half {
    nodes: Vec<Node>,
    used: u32
}

impl Half {
    pub fn new(nodes: u64) -> Self {
        let mut result = Self {
            nodes: Vec::new(),
            used: 0
        };
        result.nodes.reserve_exact(nodes as usize);
        result.nodes.resize(nodes as usize, Node::new(Move::NULL, 0.0));
        result
    }

    pub fn max_nodes(&self) -> u32 {
        self.nodes.capacity() as u32
    }

    pub fn used_nodes(&self) -> u32 {
        self.used
    }

    fn clear_indices(&mut self, half: u8) {
        for node in &mut self.nodes {
            // node's children were not copied across, clear its children to be reexpanded
            if node.first_child_idx.half() != half {
                node.first_child_idx = NodeIndex::NULL;
                node.child_count = 0;
            }
        }
    }
}

pub struct Tree {
    halves: [Half; 2],
    active_half: u8,
}

impl Tree {
    pub fn new(mb: u64) -> Self {
        let total_nodes = mb * 1024 * 1024 / std::mem::size_of::<Node>() as u64;
        let half_nodes = total_nodes / 2;
        let mut result = Self {
            halves: [Half::new(half_nodes), Half::new(half_nodes)],
            active_half: 0
        };
        result.clear();
        result
    }

    pub fn curr_half(&self) -> &Half {
        &self.halves[self.active_half as usize]
    }

    pub fn size(&self) -> u32 {
        self.curr_half().used
    }

    pub fn root_node(&self) -> NodeIndex {
        NodeIndex::new(self.active_half, 0)
    }

    pub fn clear(&mut self) {
        self.curr_half_mut().used = 1;
        self.reset_root_node();
    }

    pub fn reset_root_node(&mut self) {
        let root = self.root_node();
        self[root] = Node::new(Move::NULL, 0.0);
    }

    pub fn flip(&mut self) {
        let old_root = self.root_node();
        let half = self.active_half;
        self.curr_half_mut().clear_indices(half);
        
        self.active_half ^= 1;
        let new_root = self.root_node();
        self.curr_half_mut().used = 1;
        self.copy_node_across(old_root, new_root);
    }

    pub fn set_as_root(&mut self, node_idx: NodeIndex) {
        assert!(node_idx.half() == self.active_half);
        let root = self.root_node();
        self[root] = self[node_idx].clone();
    }

    pub fn fetch_children(&mut self, node_idx: NodeIndex) -> Option<()> {
        let old_first_child_idx = self[node_idx].first_child_idx;

        // children are already in the correct half of the tree
        if old_first_child_idx.half() == self.active_half {
            return Some(());
        }

        let new_first_child_idx = self.alloc_nodes(self[node_idx].child_count())?; 

        self.copy_nodes_across(old_first_child_idx, new_first_child_idx, self[node_idx].child_count());
        self[node_idx].first_child_idx = new_first_child_idx;

        Some(())
    }

    pub fn expand_node(&mut self, node_idx: NodeIndex, board: &Board) -> Option<()> {
        let mut moves = MoveList::new();
        movegen::movegen(board, &mut moves);

        let first_child_idx = self.alloc_nodes(moves.len() as u32)?;

        let tmp = if node_idx.index() == 0 { 3.0 } else { 1.0 };

        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;
        for mv in moves.iter() {
            let policy = policy::get_policy(board, *mv) / tmp;
            max_policy = max_policy.max(policy);
            policies.push(policy);
        }

        softmax(&mut policies, max_policy);

        let node = &mut self[node_idx];
        node.first_child_idx = first_child_idx;
        node.child_count = moves.len() as u8;

        for (i, mv) in moves.iter().enumerate() {
            let index = first_child_idx + i as u32;
            self[index] = Node::new(*mv, policies[i]);
        }

        Some(())
    }

    pub fn relabel_policies(&mut self, node_idx: NodeIndex, board: &Board) {
        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;

        let tmp = if node_idx.index() == 0 { 3.0 } else { 1.0 };

        for child_idx in self[node_idx].child_indices() {
            let policy =
                policy::get_policy(board, self[child_idx].parent_move) / tmp;
            max_policy = max_policy.max(policy);
            policies.push(policy);
        }

        softmax(&mut policies, max_policy);

        for (i, child_idx) in self[node_idx].child_indices().enumerate() {
            self[child_idx].policy = policies[i];
        }
    }

    fn copy_node_across(&mut self, old_index: NodeIndex, new_index: NodeIndex) {
        self[new_index] = self[old_index].clone();
    }

    fn copy_nodes_across(&mut self, old_index: NodeIndex, new_index: NodeIndex, count: u32) {
        for i in 0..count {
            self.copy_node_across(old_index + i, new_index + i);
        }
    }

    fn curr_half_mut(&mut self) -> &mut Half {
        &mut self.halves[self.active_half as usize]
    }

    fn alloc_nodes(&mut self, count: u32) -> Option<NodeIndex> {
        if self.curr_half().used_nodes() + count > self.curr_half().max_nodes() {
            return None;
        }
        let index = self.curr_half().used_nodes();
        self.curr_half_mut().used += count;
        Some(NodeIndex::new(self.active_half, index))
    }
}

impl Index<NodeIndex> for Tree {
    type Output = Node;
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.halves[index.half() as usize].nodes[index.index() as usize]
    }
}

impl IndexMut<NodeIndex> for Tree {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self.halves[index.half() as usize].nodes[index.index() as usize]
    }
}
