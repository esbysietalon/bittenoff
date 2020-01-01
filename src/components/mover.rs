use amethyst::ecs::prelude::{Component, VecStorage};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Goal {
    pub priority: usize,
    pub position: (u64, u64),
}

impl Goal {
    pub fn new(priority: usize, pos: (f32, f32)) -> Goal {
        Goal {
            priority,
            position: ((pos.0 * 1000.0) as u64, (pos.1 * 1000.0) as u64),
        }
    }
    pub fn pos(&self) -> (f32, f32) {
        let (x, y) = self.position;
        
        ((x as f32) / 1000.0, (y as f32) / 1000.0)
    }
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
    move_vec: Vec<(f32, f32)>,
    base_speed: f32,
    move_range: f32,
}

impl Mover{
    pub fn new(speed: f32) -> Mover {
        Mover {
            pos_goals: BinaryHeap::new(),
            move_vec: Vec::new(),
            base_speed: speed,
            move_range: 0.0,
        }
    }
    pub fn speed(&self) -> f32 {
        self.base_speed
    }
    pub fn range(&self) -> f32 {
        self.move_range
    }
    pub fn inc_range(&mut self, i: f32) {
        self.move_range += i;
    }
    pub fn is_move_vec_empty(&self) -> bool {
        self.move_vec.is_empty()
    }
    pub fn set_move_vec(&mut self, vec: Vec<(f32, f32)>) {
        //println!("setting move vec to {:?}", vec);
        self.move_vec = vec;
    }
    pub fn get_move(&self) -> Option<(f32, f32)> {
        //println!("move vec len: {}", self.move_vec.len());
        if self.move_vec.is_empty() {
            None
        }else{
            Some(self.move_vec[0])
        }
    }
    pub fn pop_move(&mut self) -> Option<(f32, f32)> {
        if self.move_vec.is_empty() {
            None
        }else{
            Some(self.move_vec.remove(0))
        }
    }
    pub fn add_goal(&mut self, goal: Goal){
        self.pos_goals.push(goal);
    }
    pub fn get_goal(&self) -> Option<(f32, f32)> {
        let goal = self.pos_goals.peek();

        match goal {
            None => None,
            Some(g) => Some(g.pos()),
        }
    }
    pub fn pop_goal(&mut self) -> Option<(f32, f32)> {
        let goal = self.pos_goals.pop();
        match goal {
            None => None,
            Some(g) => Some(g.pos()),
        }
    }
         
}

impl Component for Mover{
    type Storage = VecStorage<Self>;
}