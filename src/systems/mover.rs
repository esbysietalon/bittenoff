use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Map, Anchor};
use crate::components::{Id, Mover, Goal, Physical};

use pathfinding::prelude::astar;
use pathfinding::prelude::absdiff;

use angular::atan2;
use rand::Rng;

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut physicals, mut movers, config, time): Self::SystemData) {
        for (mover, phys) in (&mut movers, &mut physicals).join(){
            //setting position to latest move step
            //println!("next move is {:?}", mover.get_move());
            match mover.get_step() {
                Some(a) => {
                    //println!("next move {:?}", a);

                }
                None => {
                    //println!("change goal, no moves detected");
                    mover.pop_goal();
                }
            }
        }
    }
}

pub struct RudderSystem;

impl<'s> System<'s> for RudderSystem{
    type SystemData = (
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Map>,
    );

    fn run(&mut self, (mut physicals, mut movers, config, map): Self::SystemData) {
        for (mover, phys) in (&mut movers, &mut physicals).join(){
            match mover.get_goal() {
                None => {}
                Some(goal) => {
                    let (x, y) = phys.get_tile_position();
                    let origin = map.anchor_points[x + y * map.width].clone();
                    let path = astar(&origin, |p| {let successors: Vec<(Anchor, usize)> = (*p).succ.to_vec().into_iter().map(|val| (map.anchor_points[val].clone(), 1)).collect(); successors},
                    |p| (((*p).pos.0 as i32 - goal.pos.0 as i32).abs() + ((*p).pos.1 as i32 - goal.pos.1 as i32).abs()) as usize / 3,
                    |p| *p == goal);
                    //println!("path calculated!");
                    match path {
                        Some((v, c)) => {
                            //println!("path cost is {}", c);
                        }
                        None => {
                            println!("no path found!");
                        }
                    }
                }
            }
        }
    }
}

pub struct SimpleIdle;

impl<'s> System<'s> for SimpleIdle{
    type SystemData = (
        ReadStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Map>,
    );

    fn run(&mut self, (physes, mut movers, config, map): Self::SystemData) {
        for (mover, phys) in (&mut movers, &physes).join(){
            if map.location != phys.get_location() {
                continue;
            }
            
            match mover.get_goal() {
                None => {
                    let mut rng = rand::thread_rng();
                    
                    let gx = rng.gen_range(0, map.width);
                    let gy = rng.gen_range(0, map.height);

                    if gx + gy * map.width < map.anchor_points.len() {
                        mover.add_goal(Goal::new(1, map.anchor_points[gx + gy * map.width].clone()));
                    }
                },
                _ => {},
            }
        }
    }
}