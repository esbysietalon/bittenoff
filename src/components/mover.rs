use amethyst::ecs::prelude::{Component, VecStorage};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Goal {
    pub priority: usize,
    pub position: (usize, usize),
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for Goal {
    fn cmp(&self, other: &Goal) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.priority.cmp(&self.priority)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Goal {
    fn partial_cmp(&self, other: &Goal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Mover{
    pos_goals: BinaryHeap<Goal>,
    move_vec: Vec<(usize, usize)>,
}

impl Mover{
    pub fn new(width: f32, height: f32) -> Mover {
        Mover {
            pos_goals: BinaryHeap::new(),
            move_vec: Vec::new(),
        }
    }
    pub fn is_move_vec_empty(&self) -> bool {
        self.move_vec.is_empty()
    }
    pub fn set_move_vec(&mut self, vec: Vec<(usize, usize)>) {
        //println!("setting move vec to {:?}", vec);
        self.move_vec = vec;
    }
    pub fn get_move(&self) -> Option<(usize, usize)> {
        if self.move_vec.is_empty() {
            None
        }else{
            Some(self.move_vec[0])
        }
    }
    pub fn pop_move(&mut self) {
        self.move_vec.remove(0);
    }
    pub fn add_goal(&mut self, goal: Goal){
        self.pos_goals.push(goal);
    }
    pub fn get_goal(&self) -> Option<(usize, usize)> {
        let goal = self.pos_goals.peek();

        match goal {
            None => None,
            Some(g) => Some((g.position.0, g.position.1)),
        }
    }
    pub fn pop_goal(&mut self) {
        self.pos_goals.pop();
    }
         
}

impl Component for Mover{
    type Storage = VecStorage<Self>;
}