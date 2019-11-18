use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
};
use crate::game_state::{Config, Map};
use crate::components::{Physical, Id};

pub struct PhysicalSystem;

impl<'s> System<'s> for PhysicalSystem{
    type SystemData = (
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Physical>,
        ReadStorage<'s, Id>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (transforms, objs, ids, config, mut map): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        map.entities = vec![Id::nil(); (map.width * map.height) as usize];
        for (transform, obj, id) in (&transforms, &objs, &ids).join() {
            let (x, y) = obj.get_tile_position();
            let index = x + y * map.width;
                
            map.entities[index] = id.clone();
        }    
    }
}
