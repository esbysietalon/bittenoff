use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};
use crate::components::{Counter};
use crate::game_state::{Config};

use std::time::{Duration, Instant};
use std::thread::yield_now;


pub struct CounterSystem;

impl<'s> System<'s> for CounterSystem{
    type SystemData = (
        WriteStorage<'s, Counter>,
        Read<'s, Time>,
        Read<'s, Config>,
    );

    fn run(&mut self, (mut counters, time, config): Self::SystemData) {
        for counter in (&mut counters).join(){
            let time_passed = time.delta_seconds();
            counter.tick(time_passed, true);
            if counter.frames() >= config.fps_limit {
                while !counter.tick(time_passed, false) {
                    yield_now();
                }              
            }
        }
    }
}