use amethyst::ecs::prelude::{Component, VecStorage};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use crate::game_state::Anchor;

#[derive(Clone, Eq, PartialEq)]
pub struct Goal {
    pub priority: usize,
    pub point: Anchor,
}

impl Goal {
    pub fn new(priority: usize, point: Anchor) -> Goal {
        Goal {
            priority,
            point,
        }
    }
    pub fn pos(&self) -> (f32, f32) {
        self.point.real_pos()
    }

    pub fn cmp_pos(&self) -> (u32, u32) {
        let (x, y) = self.pos();
        (x as u32, y as u32)
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
            .then_with(|| self.cmp_pos().cmp(&other.cmp_pos()))
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
    step_vec: Vec<Anchor>,
    base_speed: f32,
}

impl Mover{
    pub fn new(speed: f32) -> Mover {
        Mover {
            pos_goals: BinaryHeap::new(),
            step_vec: Vec::new(),
            base_speed: speed,
        }
    }
    pub fn speed(&self) -> f32 {
        self.base_speed
    }
    pub fn is_step_vec_empty(&self) -> bool {
        self.step_vec.is_empty()
    }
    pub fn set_step_vec(&mut self, vec: Vec<Anchor>) {
        //println!("setting step vec to {:?}", vec);
        self.step_vec = vec;
    }
    pub fn get_step(&self) -> Option<Anchor> {
        //println!("step vec len: {}", self.step_vec.len());
        if self.step_vec.is_empty() {
            None
        }else{
            Some(self.step_vec[0].clone())
        }
    }
    pub fn pop_step(&mut self) -> Option<Anchor> {
        if self.step_vec.is_empty() {
            None
        }else{
            Some(self.step_vec.remove(0))
        }
    }
    pub fn add_goal(&mut self, goal: Goal){
        self.pos_goals.push(goal);
    }
    pub fn get_goal(&self) -> Option<Anchor> {
        let goal = self.pos_goals.peek();

        match goal {
            None => None,
            Some(g) => Some(g.point.clone()),
        }
    }
    pub fn pop_goal(&mut self) -> Option<Anchor> {
        let goal = self.pos_goals.pop();
        match goal {
            None => None,
            Some(g) => Some(g.point.clone()),
        }
    }
         
}

impl Component for Mover{
    type Storage = VecStorage<Self>;
}