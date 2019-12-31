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
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Physical>,
        ReadStorage<'s, Id>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (mut transforms, mut objs, ids, config, mut map): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        map.entities = vec![Id::nil(); (map.width * map.height) as usize];
        for (obj, id) in (&mut objs, &ids).join() {
            let (x, y) = obj.get_tile_position();

            let index = x + y * map.width;

            if x >= 0 && x < map.width && y >= 0 && y < map.height {
                map.entities[index] = id.clone();
            }
        }
        for (transform, obj) in (&mut transforms, &objs).join(){
            if obj.get_location() == map.location {
                transform.set_translation_xyz(obj.get_real_position().0, obj.get_real_position().1, 0.0);
            }else{
                transform.set_translation_xyz(-100.0, 0.0, 0.0);
            }
            
        }
    }
}
