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
        self.point.real_local()
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
    path_cost: usize,
    base_speed: f32,
    last_step: (usize, usize),
}

impl Mover{
    pub fn new(speed: f32) -> Mover {
        Mover {
            pos_goals: BinaryHeap::new(),
            step_vec: Vec::new(),
            path_cost: 0,
            base_speed: speed,
            last_step: (usize::max_value(), usize::max_value()),
        }
    }
    pub fn speed(&self) -> f32 {
        self.base_speed
    }
    pub fn is_step_vec_empty(&self) -> bool {
        self.step_vec.is_empty()
    }
    pub fn clear_step_vec(&mut self) {
        self.step_vec.clear();
        self.path_cost = 0;
    }
    pub fn set_step_vec(&mut self, vec: Vec<Anchor>, cost: usize) {
        //println!("setting step vec to {:?}", vec);
        self.step_vec = vec;
        self.path_cost = cost;
    }
    pub fn get_step(&self) -> Option<Anchor> {
        //println!("step vec len: {}", self.step_vec.len());
        if self.step_vec.is_empty() {
            None
        }else{
            Some(self.step_vec[0].clone())
        }
    }
    pub fn path_cost(&self) -> usize {
        self.path_cost
    }
    pub fn path(&self) -> Vec<Anchor> {
        self.step_vec.clone()
    }
    pub fn pop_step(&mut self) -> Option<Anchor> {
        if self.step_vec.is_empty() {
            None
        }else{
            let mut cost = 10;
            if self.last_step != (usize::max_value(), usize::max_value()) {
                if self.last_step.0 != self.step_vec[0].pos.0 && self.last_step.1 != self.step_vec[1].pos.1 {
                    cost = 14;
                }
            }
            if self.path_cost >= cost {
                self.path_cost -= cost;
            }else{
                self.path_cost = 0;
            }
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