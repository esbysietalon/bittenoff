use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
};
use crate::game_state::{Config, Map, TILE_SIZE, OFFSCREEN_UNKNOWN_PATH_WAIT_TIME};
use crate::components::{Physical, Id, Mover, Offscreen};

pub struct PhysicalSystem;

impl<'s> System<'s> for PhysicalSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        WriteStorage<'s, Offscreen>,
        ReadStorage<'s, Id>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (mut transforms, mut objs, mut movers, mut offscreens, ids, config, mut map): Self::SystemData) {
        //TODO read objs and transforms and apply to map
        map.entities = vec![Id::nil(); (map.width * map.height) as usize];
        for (obj, id, mover) in (&mut objs, &ids, &mut movers).join() {
            if obj.get_location() == map.location {
                let (x, y) = obj.get_tile_position();

                let index = x + y * map.width;

                if x < map.width && y < map.height {
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
        for (phys, mover, offs) in (&mut objs, &mut movers, &mut offscreens).join() {
            
            let offs_time = offs.time_passed();
            if offs_time > 0.0 {
                let p_c = mover.path_cost();

                let mut path_completed = offs_time / OFFSCREEN_UNKNOWN_PATH_WAIT_TIME;
                
                if p_c > 0 {
                    //println!("have path");
                    let real_p_c = (p_c * TILE_SIZE) as f32 / 10.0;
                    let dist_traveled = mover.speed() * offs_time;
                    path_completed = dist_traveled / real_p_c;
                }

                //println!("path_completed {}", path_completed);

                if path_completed >= 1.0 {
                    match mover.get_goal() {
                        Some(anchor) => {
                            if anchor.area() != phys.get_location() {
                                let (ax, ay) = anchor.area();
                                let (px, py) = phys.get_location();

                                if px < ax {
                                    //east
                                    //println!("east");
                                    phys.mut_area_x(1);
                                    phys.set_x(TILE_SIZE as f32 / 2.0);  
                                }else if px > ax {
                                    //west
                                    //println!("west");
                                    phys.mut_area_x(-1);
                                    phys.set_x(config.stage_height - TILE_SIZE as f32 / 2.0);
                                }else if py < ay {
                                    //north
                                    //println!("north");
                                    phys.mut_area_y(1);
                                    phys.set_y(TILE_SIZE as f32 / 2.0);
                                }else if py > ay {
                                    //south
                                    //println!("south");
                                    phys.mut_area_y(-1);
                                    phys.set_y(config.stage_height - TILE_SIZE as f32 / 2.0);
                                }
                            } else {
                                let (gx, gy) = anchor.real_local();
                                //within area
                                //println!("within area");
                                phys.set_x(gx);
                                phys.set_y(gy);
                                mover.pop_goal();
                            }
                            mover.clear_step_vec();
                        }
                        None => {}
                    }
                    offs.reset();
                }else if path_completed > 0.0 {
                    if phys.get_location() == map.location {
                        let path = mover.path();
                        
                        if path.len() > 0 {
                            let index = (path_completed * path.len() as f32) as usize;

                            //println!("on path at index {} of {}", index, path.len());

                            let (cx, cy) = path[index].real_local();

                            phys.set_x(cx);
                            phys.set_y(cy);
                        }
                        offs.reset();
                    }
                }

            }
        }
        for (transform, obj) in (&mut transforms, &objs).join(){

            //println!("obj location is now {:?}", (obj.get_tile_position(), obj.get_location()));
            if obj.get_location() == map.location {
                transform.set_translation_xyz(obj.get_real_position().0, obj.get_real_position().1, 0.0);
            }else{
                transform.set_translation_xyz(-100.0, 0.0, 0.0);
            }
            
        }
    }
}
