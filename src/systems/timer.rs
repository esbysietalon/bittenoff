use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
};
use crate::game_state::{Timer, TICK_RATE};

pub struct TimerSystem;

impl<'s> System<'s> for TimerSystem{
    type SystemData = (
        Write<'s, Timer>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut timer, time): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        if timer.time >= TICK_RATE {
            timer.time = 0.0;
            timer.tick = true;
        }else {
            timer.time += time.delta_seconds();
            timer.tick = false;
        }
    }
}
