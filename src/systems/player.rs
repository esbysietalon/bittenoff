use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config};
use crate::components::{Player};

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, players, config, input, time): Self::SystemData) {
        for (player, transform) in (&players, &mut transforms).join(){
            let movement = input.axis_value("horizontal_mv");
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let x = transform.translation().x;
                transform.set_translation_x(
                    (x + scaled_amount)
                        .min(config.stage_width - player.width * 0.5)
                        .max(player.width * 0.5),
                );
            }


            let movement = input.axis_value("vertical_mv");
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let y = transform.translation().y;
                transform.set_translation_y(
                    (y + scaled_amount)
                        .min(config.stage_height - player.height * 0.5)
                        .max(player.height * 0.5),
                );
            }
        }
    }
}