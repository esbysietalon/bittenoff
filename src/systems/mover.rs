use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Map};
use crate::components::{Id, Mover, Goal};

use pathfinding::prelude::astar;

use angular::atan2;
use rand::Rng;

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){
        }
    }
}

pub struct RudderSystem;

impl<'s> System<'s> for RudderSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){

        }
    }
}

pub struct SimpleIdle;

impl<'s> System<'s> for SimpleIdle{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){
            match mover.get_goal() {
                None => {
                    let mut rng = rand::thread_rng();
                    //println!("adding goal");
                    mover.add_goal(Goal { position: (rng.gen_range(0.0, config.stage_width) as usize, rng.gen_range(0.0, config.stage_height) as usize), priority: 1});
                },
                _ => {},
            }
        }
    }
}