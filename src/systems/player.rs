use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::components::{Player, Physical};
use crate::game_state::{Config, Map, regenerate_map};

pub struct MapSystem;

impl<'s> System<'s> for MapSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (players, mut transforms, config, mut map): Self::SystemData) {
        let mut change_map = false;
        for (player, transform) in (&players, &mut transforms).join(){
            let x = transform.translation().x;
            let y = transform.translation().y;

            if x > config.stage_width as f32 {
                transform.set_translation_x(0.0);
                change_map = true;
            }else if x < 0.0 {
                transform.set_translation_x(config.stage_width as f32);
                change_map = true;
            }else if y > config.stage_height as f32 {
                transform.set_translation_y(0.0);
                change_map = true;
            }else if y < 0.0 {
                transform.set_translation_y(config.stage_height as f32);
                change_map = true;
            }
        }

        if change_map {
            println!("change map!");
            regenerate_map(&mut map);
        }
    }
}

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>
    );

    fn run(&mut self, (players, mut transforms, config, input, time): Self::SystemData) {
        

        for (player, transform) in (&players, &mut transforms).join(){
            let movement = input.axis_value("horizontal_mv");
            //println!("running, since {}", time.delta_seconds());
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let x = transform.translation().x;
                //println!("updating x by {}", scaled_amount);
                transform.set_translation_x(
                    (x + scaled_amount)
                        //.min(config.stage_width as f32 - player.width as f32 * 0.5)
                        //.max(player.width as f32 * 0.5),
                );
            }


            let movement = input.axis_value("vertical_mv");
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let y = transform.translation().y;
                //println!("updating x by {}", scaled_amount);
                transform.set_translation_y(
                    (y + scaled_amount)
                        //.min(config.stage_height as f32 - player.height as f32 * 0.5)
                        //.max(player.height as f32 * 0.5),
                );
            }
        }
    }
}