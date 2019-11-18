use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Timer};
use crate::components::{Player, Physical};

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Physical>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Timer>
    );

    fn run(&mut self, (players, mut physicals, config, input, timer): Self::SystemData) {
        //println!("timer details {} {}", timer.time, timer.tick);
        if timer.tick() {
            for (player, physical) in (&players, &mut physicals).join(){
                println!("player position at {:?}", physical.get_tile_position());

                let movement = input.axis_value("horizontal_mv");
                if let Some(mv_amount) = movement {
                    if mv_amount < 0.0 {
                        physical.move_tile("l");
                    } else if mv_amount > 0.0 {
                        physical.move_tile("r");
                    }
                }


                let movement = input.axis_value("vertical_mv");
                if let Some(mv_amount) = movement {
                    if mv_amount < 0.0 {
                        physical.move_tile("d");
                    } else if mv_amount > 0.0 {
                        physical.move_tile("u");
                    }
                }
            }
        }
    }
}