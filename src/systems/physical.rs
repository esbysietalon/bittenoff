use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
};
use crate::game_state::{Config, Map, TILE_SIZE};
use crate::components::{Physical, Id};

pub struct PhysicalSystem;

impl<'s> System<'s> for PhysicalSystem{
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Physical>,
        ReadStorage<'s, Id>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (transforms, mut objs, ids, config, mut map): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        map.entities = vec![Id::nil(); (map.width * map.height) as usize];
        for (transform, obj, id) in (&transforms, &mut objs, &ids).join() {
            let (x, y) = obj.set_tile_position((transform.translation().x, transform.translation().y));

            let index = x + y * map.width;

            if x >= 0 && x < map.width && y >= 0 && y < map.height {
                map.entities[index] = id.clone();
            }
        }
    }
}
