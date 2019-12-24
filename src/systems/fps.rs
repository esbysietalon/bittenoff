use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};
use crate::components::{Counter};
use crate::game_state::{Config};

pub struct CounterSystem;

impl<'s> System<'s> for CounterSystem{
    type SystemData = (
        WriteStorage<'s, Counter>,
        Read<'s, Time>
    );

    fn run(&mut self, (mut counters, time): Self::SystemData) {
        for counter in (&mut counters).join(){
            counter.tick(time.delta_seconds());
        }
    }
}