use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
};
use crate::game_state::{Map};
use crate::components::{Hunger, Mover};

pub struct HungerSystem;

impl<'s> System<'s> for HungerSystem{
    type SystemData = (
        WriteStorage<'s, Hunger>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut hungs, time): Self::SystemData) {
        for hung in (&mut hungs).join() {
            hung.mut_hunger(-1.0 * hung.get_rate() * time.delta_seconds());
        }
    }
}

pub struct GoalSystem;

impl<'s> System<'s> for GoalSystem{
    type SystemData = (
        WriteStorage<'s, Hunger>,
        WriteStorage<'s, Mover>,
        Read<'s, Time>,
        Read<'s, Map>,
    );

    fn run(&mut self, (mut hungs, mut movers, time, map): Self::SystemData) {
        for (hung, mover) in (&mut hungs, &mut movers).join() {
            if (hung.get_hunger() < hung.get_capacity() / 0.3) || (hung.get_hunger() < hung.get_rate() * 240.0) {
                //hungry: go find meal

                //search for meal in local area

                //if no meal found in local area, search for meal in other area
            }
        }
    }
}