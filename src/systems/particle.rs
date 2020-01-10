use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use amethyst::ecs::prelude::Entities;
use crate::game_state::{Config, UiHolder};
use crate::components::{Particle, ParticleDeathType};

pub struct ParticleCleanUpSystem;

impl<'s> System<'s> for ParticleCleanUpSystem{
    type SystemData = (
        WriteStorage<'s, Particle>,
        Entities<'s>,
        Read<'s, Config>,
        Read<'s, UiHolder>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut parts, mut ents, config, ui_holder, time): Self::SystemData) {
        for (part, ent) in (&mut parts, &*ents).join() {
            match part.get_death_type() {
                ParticleDeathType::Ui => {
                    if !ui_holder.is_active(part.get_ui()) {
                        ents.delete(ent);
                    }
                }
                _ => {

                }
            }
        }
    }
}
pub struct ParticleDisplaySystem;

impl<'s> System<'s> for ParticleDisplaySystem{
    type SystemData = (
        WriteStorage<'s, Particle>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut parts, mut trans): Self::SystemData) {
        for (part, tran) in (&mut parts, &mut trans).join() {
            tran.set_translation_xyz(part.get_x(), part.get_y(), 1.1);
        }
    }
}