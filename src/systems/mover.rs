use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Map};
use crate::components::{Mover, Physical, Id, Priority};
use crate::navigation::{find_path, InputData, successors, distance, heuristic};

use pathfinding::prelude::astar;


use angular::atan2;
use rand::Rng;

pub struct NavigationSystem;

impl NavigationSystem{
    pub fn simplify_path(path: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        
        //println!("{:?}", path);

        let mut last_dx = 0.0;
        let mut last_dy = 0.0;
        let mut last_slope = 0.0;
        
        let mut last_pos = path[0];
        for (x, y) in path {
            let new_dx = (x - last_pos.0) as f32;
            let new_dy = (y - last_pos.1) as f32;
            let out_len = out.len();
            
            if last_dx == 0.0 {
                if new_dx != 0.0 {
                    out.push((x, y));
                    last_slope = new_dy as f32 / new_dx as f32;    
                }else{
                    if out_len > 0 {
                        out[out_len - 1] = (x, y);
                    }
                }
            }else if new_dx == 0.0 {
                if last_dx != 0.0 {
                    out.push((x, y));
                }else{
                    if out_len > 0 { 
                        out[out_len - 1] = (x, y);
                    }
                }
            }else{
                let new_slope = new_dy as f32 / new_dx as f32;
                if last_slope != new_slope {
                    out.push((x, y));
                }else{
                    if out_len > 0 { 
                        out[out_len - 1] = (x, y);
                    }
                }
                last_slope = new_slope;
            }
            last_dx = new_dx;
            last_dy = new_dy;
            last_pos = (x, y);
        }
        
        //println!("{:?}", out);
        out
    }
}

impl<'s> System<'s> for NavigationSystem{
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Physical>,
        ReadStorage<'s, Id>,
        Write<'s, Map>,
        Read<'s, Time>,
    );
    fn run(&mut self, (transforms, mut movers, physicals, ids, mut map, time): Self::SystemData) {
        let mut mover_index = 0;
        for (transform, mover, physical, id) in (&transforms, &mut movers, &physicals, &ids).join() {
            //let mut local_copy = map.clone();
            match mover.get_goal() {
                None => mover.set_move_vec(Vec::new()),
                Some(e) => {
                    if mover.is_move_vec_empty() {

                        let x = transform.translation().x as i32;
                        let y = transform.translation().y as i32;

                        let index = (x + y * map.width as i32) as usize;

                        if !(x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32) {
                            map.storage[index] = (1.0, *id);
                        }

                        let pathfind_input = InputData {
                            id: id.clone(),
                            obj: physical.clone(),
                            map: map.clone(),
                        };

                        let result = astar(&(x, y), |p| successors(p, id, physical, &*map), |p| heuristic(p, &e),
                        |p| *p == e);

                        //let result = find_path((100, 100), (200, 200), pathfind_input);

                        //let result = find_path((x, y), e, pathfind_input);

                        match result {
                            None => {
                                //println!("no path found from {:?} to {:?}", (transform.translation().x, transform.translation().y), e);
                                //println!("discarding goal..");
                                mover.pop_goal();
                            }
                            Some((mut path, cost)) => {
                                //println!("found path with cost {}", cost);
                                path = NavigationSystem::simplify_path(path);
                                mover.set_move_vec(path);
                            }
                        }
                        
                        mover.pop_goal();
                    }else{
                        //TODO revisit in-vector updating of movement
                        //code below breaks stuff - consider using cheap astar to the next point maybe(?)
                        //also may be issues with how I'm replacing the move vector
                        mover.session += time.delta_seconds();
                        if mover.session >= 0.5 {
                        
                            //let m = mover.get_move().unwrap();

                            let x = transform.translation().x as i32;
                            let y = transform.translation().y as i32;

                            let index = (x + y * map.width as i32) as usize;

                            if !(x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32) {
                                map.storage[index] = (1.0, *id);
                            }
                            

                            let result = astar(&(x, y), |p| successors(p, id, physical, &*map), |p| heuristic(p, &e),
                            |p| *p == e);
                            
                            match result {
                                None => {
                                    //println!("no path found from {:?} to {:?}", (transform.translation().x, transform.translation().y), e);
                                    //println!("discarding goal..");
                                    mover.pop_goal();
                                }
                                Some((mut path, cost)) => {
                                    //println!("found path with cost {}", cost);
                                    path = NavigationSystem::simplify_path(path);
                                    //println!("path is {:?}", path);
                                    /*if path.len() > 0 {
                                        path.remove(0);
                                    }*/
                                    
                                    //let path_diff = mover.diff_move_vec(&path);
                                    
                                    //println!("path diff is {:?}", path_diff);
                                    //if path_diff.len() > 0 {
                                    //    println!("Different path detected.. Replacing");
                                    mover.set_move_vec(path);
                                    //}
                                }
                            }

                            
                            //let mut rng = rand::thread_rng();
                            mover.session = 0.01 * mover_index as f32;
                        
                        }

                    }
                }
            };

            mover_index += 1;
        }
    }
}

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){
            //println!("x-vel: {} y-vel: {}", mover.velocity[0], mover.velocity[1]);
            
            //mover.acceleration[0] *= 0.95;
            //mover.acceleration[1] *= 0.95;
            mover.velocity[0] += mover.acceleration[0];
            mover.velocity[1] += mover.acceleration[1];


            mover.velocity[0] *= 0.2;
            mover.velocity[1] *= 0.2;
            /*if !mover.is_move_vec_empty(){
                transform.set_translation_x(mover.get_move().unwrap().0 as f32);
                transform.set_translation_y(mover.get_move().unwrap().1 as f32);
                mover.pop_move();
            }*/

            //mover.velocity[0] = mover.velocity[0].round();
            //mover.velocity[1] = mover.velocity[1].round();
            
            let use_vel_x = mover.velocity[0] * time.delta_seconds();//.round();
            let use_vel_y = mover.velocity[1] * time.delta_seconds();//.round();

            //println!("vel [{}, {}] acc [{}, {}]", use_vel_x, use_vel_y, mover.acceleration[0], mover.acceleration[1]);


            

            transform.prepend_translation_x(use_vel_x);
            transform.prepend_translation_y(use_vel_y);
            
        }
    }
}

pub struct RudderSystem;

impl<'s> System<'s> for RudderSystem{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){
            let step = mover.get_move();
            let px = transform.translation().x;
            let py = transform.translation().y;
            //println!("curr pos ({}, {})", px, py);
            match step {
                None => {
                    //println!("no move, attempting stop");
                    mover.acceleration[0] += (-mover.jerk).max(mover.jerk.min(-mover.velocity[0]));
                    mover.acceleration[1] += (-mover.jerk).max(mover.jerk.min(-mover.velocity[1]));
                },
                Some((x, y)) => {
                    //println!("curr move ({}, {})", x, y);

                    let use_x = x - px as i32;
                    let use_y = y - py as i32;
                    
                    let use_dist = ((use_x * use_x + use_y * use_y) as f32).sqrt();
                    
                    //println!("dist {}", use_dist);

                    if use_dist <= 10.0 {
                        //println!("close enough, popping move");
                        
                        mover.pop_move();
                    }else{
                        let ang = atan2(use_y as f32, use_x as f32);

                        let aim_vel_x = ang.cos() * (500.0 as f32).min(use_dist.max(200.0));
                        let aim_vel_y = ang.sin() * (500.0 as f32).min(use_dist.max(200.0));

                        //println!("aim velocity is ({}, {})", aim_vel_x, aim_vel_y);

                        //mover.velocity[0] = aim_vel_x;
                        //mover.velocity[1] = aim_vel_y;

                        let aim_acc_x = aim_vel_x - mover.velocity[0];
                        let aim_acc_y = aim_vel_y - mover.velocity[1];

                        let accelx = (-mover.jerk).max(mover.jerk.min(aim_acc_x - mover.acceleration[0]));//mover.jerk.min((-mover.jerk).max(aim_acc_x));
                        let accely = (-mover.jerk).max(mover.jerk.min(aim_acc_y - mover.acceleration[1]));//mover.jerk.min((-mover.jerk).max(aim_acc_y));

                        //println!("acceleration is ({}, {})", accelx, accely);
                        mover.acceleration[0] += accelx;
                        mover.acceleration[1] += accely;
                        
                        /*
                        if mover.velocity[0].abs() > use_x.abs() as f32 && mover.velocity[0] * use_x as f32 > 0.0 {
                            println!("x attempting to avoid overshoot");
                            let aim = use_x as f32 - mover.velocity[0];
                            mover.acceleration[0] += mover.jerk.max((-mover.jerk).min(aim - mover.acceleration[0]));
                        }else{
                            println!("x normal acceleration");
                            mover.acceleration[0] += ang.cos() * time.delta_seconds() * use_dist.max(mover.jerk * 40.0);
                        }

                        if mover.velocity[1].abs() > use_y.abs() as f32  && mover.velocity[1] * use_y as f32 > 0.0 {
                            println!("y attempting to avoid overshoot");
                            let aim = use_y as f32 - mover.velocity[1];
                            mover.acceleration[1] += mover.jerk.max((-mover.jerk).min(aim - mover.acceleration[1]));
                        }else{
                            println!("y normal acceleration");
                            mover.acceleration[1] += ang.sin() * time.delta_seconds() * use_dist.max(mover.jerk * 40.0);
                        }*/
                    }
                },
            }
        }
    }
}

pub struct SimpleIdle;

impl<'s> System<'s> for SimpleIdle{
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut movers, config, time): Self::SystemData) {
        for (mover, transform) in (&mut movers, &mut transforms).join(){
            match mover.get_goal() {
                None => {
                    let mut rng = rand::thread_rng();
                    //println!("adding goal");
                    mover.add_goal((rng.gen_range(0.0, config.stage_width) as i32, rng.gen_range(0.0, config.stage_height) as i32), Priority::Low);
                },
                _ => {},
            }
        }
    }
}