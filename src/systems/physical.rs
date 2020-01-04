use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
};
use crate::game_state::{Config, Map, TILE_SIZE};
use crate::components::{Physical, Id, Mover};

pub struct PhysicalSystem;

impl<'s> System<'s> for PhysicalSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Id>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (mut transforms, mut objs, mut movers, ids, config, mut map): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        map.entities = vec![Id::nil(); (map.width * map.height) as usize];
        for (obj, id, mover) in (&mut objs, &ids, &mut movers).join() {
            if obj.get_location() == map.location {
                let (x, y) = obj.get_tile_position();

                let index = x + y * map.width;

                if x >= 0 && x < map.width && y >= 0 && y < map.height {
                    map.entities[index] = id.clone();
                }

                let (x, y) = obj.get_real_position();
                let mut area_changed = false;

                

                if x > config.stage_width as f32 {
                    //println!("obj position is {:?}; stage dim are {:?}", (x, y), (config.stage_width, config.stage_height));
                    //println!("obj goes east");
                    obj.mut_area_x(1);
                    area_changed = true;
                    obj.set_x(TILE_SIZE as f32 / 2.0);    
                }else if x < 0.0 {
                    //println!("obj position is {:?}; stage dim are {:?}", (x, y), (config.stage_width, config.stage_height));
                    //println!("obj goes west");
                    obj.mut_area_x(-1);
                    area_changed = true;
                    obj.set_x(config.stage_width - TILE_SIZE as f32 / 2.0);
                }else if y > config.stage_height as f32 {
                    //println!("obj position is {:?}; stage dim are {:?}", (x, y), (config.stage_width, config.stage_height));
                    //println!("obj goes north");
                    obj.mut_area_y(1);
                    area_changed = true;
                    obj.set_y(TILE_SIZE as f32 / 2.0);
                }else if y < 0.0 {
                    //println!("obj position is {:?}; stage dim are {:?}", (x, y), (config.stage_width, config.stage_height));
                    //println!("obj goes south");
                    obj.mut_area_y(-1);
                    area_changed = true;
                    obj.set_y(config.stage_height - TILE_SIZE as f32 / 2.0);
                }

                if area_changed {
                    //println!("obj area is now: {:?} (local position is {:?}) tryna get to {:?}", obj.get_location(), obj.get_real_position(), mover.get_goal());
                    mover.clear_step_vec();
                }
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
