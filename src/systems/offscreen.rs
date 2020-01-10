use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Map, Anchor};
use crate::components::{Id, Mover, Goal, Physical, Offscreen};

pub struct OffscreenSystem;

impl<'s> System<'s> for OffscreenSystem{
    type SystemData = (
        WriteStorage<'s, Offscreen>,
        WriteStorage<'s, Physical>,
        Read<'s, Map>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut offscreens, mut physicals, map, config, time): Self::SystemData) {
        for (phys, offs) in (&mut physicals, &mut offscreens).join() {
            if phys.get_location() != map.location { 
                //if offscreen, increase offscreen time
                offs.tick(time.delta_seconds());
            }else{
                offs.reset();
            }
        }
    }
}