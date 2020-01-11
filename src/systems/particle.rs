use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use amethyst::ecs::prelude::Entities;
use crate::game_state::{Config, UiHolder, UiState};
use crate::components::{Particle, ParticleDeathType};

pub struct ParticleCleanUpSystem;

impl<'s> System<'s> for ParticleCleanUpSystem{
    type SystemData = (
        WriteStorage<'s, Particle>,
        Entities<'s>,
        Read<'s, Config>,
        Read<'s, UiHolder>,
        Write<'s, UiState>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut parts, mut ents, config, ui_holder, mut ui_state, time): Self::SystemData) {
        for (part, ent) in (&mut parts, &*ents).join() {
           let mut deleted = false;
            match part.get_death_type() {
                ParticleDeathType::Ui => {
                    if !deleted && !ui_holder.is_active(part.get_ui()) {
                        ents.delete(ent);
                        deleted = true;
                    }
                }
                _ => {

                }
            }
            match part.get_key_check() {
                None => {}
                Some(key) => {
                    if !deleted && ui_state.key_check[key] {
                        ents.delete(ent);
                        deleted = true;
                    }
                }
            }
            if !deleted && part.get_lifespan() <= 0.0 {
                ents.delete(ent);
                deleted = true;
            }
            part.mut_lifespan(-time.delta_seconds());
        }
        for i in 0..ui_state.key_check.len() {
            if ui_state.key_check[i] {
                ui_state.key_check[i] = false;
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