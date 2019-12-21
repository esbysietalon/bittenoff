use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::components::{Player, Physical};
use crate::game_state::{Config};

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
            println!("running, since {}", time.delta_seconds());
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let x = transform.translation().x;
                println!("updating x by {}", scaled_amount);
                transform.set_translation_x(
                    (x + scaled_amount)
                        .min(config.stage_width as f32 - player.width as f32 * 0.5)
                        .max(player.width as f32 * 0.5),
                );
            }


            let movement = input.axis_value("vertical_mv");
            if let Some(mv_amount) = movement {
                let scaled_amount = 100.0 * time.delta_seconds() * mv_amount as f32;
                let y = transform.translation().y;
                println!("updating x by {}", scaled_amount);
                transform.set_translation_y(
                    (y + scaled_amount)
                        .min(config.stage_height as f32 - player.height as f32 * 0.5)
                        .max(player.height as f32 * 0.5),
                );
            }
        }
    }
}