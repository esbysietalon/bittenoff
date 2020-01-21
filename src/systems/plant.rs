use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    renderer::SpriteRender,
};
use crate::game_state::{Map, Plant as PlantSprite};
use crate::components::{Plant, Offscreen};

pub struct PlantSystem;

impl<'s> System<'s> for PlantSystem{
    type SystemData = (
        WriteStorage<'s, Plant>,
        ReadStorage<'s, Offscreen>,
        WriteStorage<'s, SpriteRender>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut plants, offs, mut srs, time): Self::SystemData) {
        for (plant, off) in (&mut plants, &offs).join() {
            if plant.get_fruit_progress() < 1.0 && plant.get_fruiting() {
                plant.mut_fruit_progress(plant.get_fruit_rate() * time.delta_seconds());
                plant.mut_fruit_progress(plant.get_fruit_rate() * off.time_passed());
            }
        }
        for (plant, sr) in (&mut plants, &mut srs).join() {
            if plant.get_fruit_progress() >= 1.0 {
                sr.sprite_number = PlantSprite::BushBerryRipe as usize;
            }else{
                sr.sprite_number = PlantSprite::BushBerryNoBerry as usize;
            }
        }
    }
}