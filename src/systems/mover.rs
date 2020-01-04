use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::game_state::{Config, Map, Anchor};
use crate::components::{Id, Mover, Goal, Physical};

use pathfinding::prelude::astar;
use pathfinding::prelude::absdiff;

use angular::atan2;
use rand::Rng;

pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut physicals, mut movers, config, time): Self::SystemData) {
        for (mover, phys) in (&mut movers, &mut physicals).join(){
            //setting position to latest move step
            //println!("next move is {:?}", mover.get_move());
            match mover.get_step() {
                Some(a) => {
                    //println!("next move: {:?} curr goal: {:?}", a.pos, mover.get_goal());
                    if a.area() == phys.get_location() {
                        if a.local() == phys.get_tile_position() {
                            //println!("reached goal successfully!");
                            mover.pop_step();
                        }
                        let (ox, oy) = phys.get_real_position();
                        let (ex, ey) = a.real_local();
                        if ox < ex {
                            phys.mut_x(mover.speed() * time.delta_seconds()); 
                        }else if ox > ex {
                            phys.mut_x(-mover.speed() * time.delta_seconds());
                        }
                        if oy < ey {
                            phys.mut_y(mover.speed() * time.delta_seconds());
                        }else if oy > ey {
                            phys.mut_y(-mover.speed() * time.delta_seconds());
                        }
                    }else{
                        println!("wrong area, recalculate path");
                        mover.clear_step_vec();
                    }
                }
                None => {
                    //println!("change goal, no moves detected");
                    //mover.pop_goal();
                }
            }
        }
    }
}

pub struct RudderSystem;

impl<'s> System<'s> for RudderSystem{
    type SystemData = (
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Map>,
    );

    fn run(&mut self, (mut physicals, mut movers, config, map): Self::SystemData) {
        for (mover, phys) in (&mut movers, &mut physicals).join(){
            if mover.is_step_vec_empty() && map.location == phys.get_location() {
                match mover.get_goal() {
                    None => {}
                    Some(goal) => {    
                        let (x, y) = phys.get_tile_position();
                        //println!("current tile position {:?}", (x, y));
                        
                        //println!("phys {:?} goal {:?}", (phys.get_tile_position(), phys.get_location()), (goal.local(), goal.area()));
                        if (x, y) != goal.local() && phys.get_location() == goal.area() {

                            let origin = map.anchor_points[x + y * map.width + 4].clone();
                            //println!("anchor points are currently: {:?}", map.anchor_points);
                            let path = astar(&origin, |p| {let successors: Vec<(Anchor, usize)> = (*p).succ.to_vec().into_iter().map(|val| (map.anchor_points[val.0].clone(), val.1)).collect(); successors},
                            |p| (((*p).pos.0 as i32 - goal.pos.0 as i32).abs() + ((*p).pos.1 as i32 - goal.pos.1 as i32).abs()) as usize / 3,
                            |p| *p == goal);
                            //println!("path calculated!");
                            match path {
                                Some((v, c)) => {
                                    //println!("path cost is {}", c);
                                    //println!("same area: path is {:?}", v);
                                    mover.set_step_vec(v);
                                }
                                None => {
                                    if map.location == phys.get_location() {
                                        println!("same area no path found! from {:?} to {:?}", (x, y, phys.get_location().0, phys.get_location().1), goal);
                                        mover.pop_goal();
                                    }
                                }
                            }
                        }else if phys.get_tile_position() == goal.local() && phys.get_location() == goal.area() {
                            //println!("reached goal successfully");
                            mover.pop_goal();
                        }else if goal.area() != phys.get_location() {
                            //println!("not same area, sending to other area");
                            //println!("not same area goal is {:?}", goal);

                            let (cax, cay) = phys.get_location();
                            let (gax, gay) = goal.area();

                            let mut goal = Anchor::new(0, 0, 0, 0);
                            
                            /*
                            area.anchor_points.push(west);
                            area.anchor_points.push(east);
                            area.anchor_points.push(north);
                            area.anchor_points.push(south);
                            */
                            /*if map.location == phys.get_location() {
                                println!("curr locale: {:?} need to get to: {:?}", (cax, cay), (gax, gay));
                            }*/
                            if cax < gax {
                                //east
                                //println!("go east");
                                goal = map.anchor_points[1].clone();
                            }else if cax > gax {
                                //west
                               //println!("go west");
                                goal = map.anchor_points[0].clone();
                            }else if cay < gay {
                                //north
                                //println!("go north");
                                goal = map.anchor_points[2].clone();
                            }else if cay > gay {
                                //south
                                //println!("go south");
                                goal = map.anchor_points[3].clone();
                            }

                            //println!("goal anchor is {:?}", goal);

                            let origin = map.anchor_points[x + y * map.width + 4].clone();
                            let path = astar(&origin, |p| {let successors: Vec<(Anchor, usize)> = (*p).succ.to_vec().into_iter().map(|val| (map.anchor_points[val.0].clone(), val.1)).collect(); successors},
                            |p| (((*p).pos.0 as i32 - goal.pos.0 as i32).abs() + ((*p).pos.1 as i32 - goal.pos.1 as i32).abs()) as usize / 3,
                            |p| *p == goal);
                            //println!("path calculated!");
                            match path {
                                Some((v, c)) => {
                                    //println!("path cost is {}", c);
                                    //println!("change area: path is {:?}", v);
                                    mover.set_step_vec(v);
                                }
                                None => {
                                    if phys.get_location() == map.location {
                                        //println!("change area no path found! from {:?} to {:?}", (x, y, phys.get_location().0, phys.get_location().1), goal);
                                        mover.pop_goal();
                                    }
                                }
                            }  
                        }
                    }
                }
            }
        }
    }
}

pub struct SimpleIdle;

impl<'s> System<'s> for SimpleIdle{
    type SystemData = (
        ReadStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Map>,
    );

    fn run(&mut self, (physes, mut movers, config, map): Self::SystemData) {
        for (mover, phys) in (&mut movers, &physes).join(){
            if map.location != phys.get_location() {
                continue;
            }
            
            match mover.get_goal() {
                None => {
                    let mut rng = rand::thread_rng();
                    
                    let ax = rng.gen_range(-1, 2);
                    let ay = rng.gen_range(-1, 2);

                    let gx = rng.gen_range(0, map.width);
                    let gy = rng.gen_range(0, map.height);

                    let index = gx + gy * map.width;
                    //if (ax, ay) == map.location {
                    if index < map.width * map.height && index < map.anchor_points.len() {
                        //println!("adding goal {:?}", map.anchor_points[gx + gy * map.width].pos);
                        let mut dest_anchor = map.anchor_points[gx + gy * map.width + 4].clone();
                        dest_anchor.set_area((ax, ay));

                        //println!("destination anchor {:?}", dest_anchor);
                        mover.add_goal( Goal::new(1, dest_anchor) );
                    }
                    /*}else{
                        let (mut tempx, mut tempy) = map.location;
                        
                        /*
                        area.anchor_points.push(west);
                        area.anchor_points.push(east);
                        area.anchor_points.push(north);
                        area.anchor_points.push(south);
                        */

                        while (tempx, tempy) != (ax, ay) {
                            if tempx < ax {
                                //east
                                let mut east_anchor = map.anchor_points[1].clone();
                                east_anchor.set_area((tempx, tempy));
                                mover.add_goal( Goal::new(1, east_anchor) );
                                tempx += 1;
                            }else if tempx > ax {
                                //east
                                let mut west_anchor = map.anchor_points[0].clone();
                                west_anchor.set_area((tempx, tempy));
                                mover.add_goal( Goal::new(1, west_anchor) );
                                tempx -= 1;
                            }
                            if tempy < ay {
                                //north
                                let mut north_anchor = map.anchor_points[2].clone();
                                north_anchor.set_area((tempx, tempy));
                                mover.add_goal( Goal::new(1, north_anchor) );
                                tempy += 1;
                            }else if tempy > ay {
                                //south
                                let mut south_anchor = map.anchor_points[3].clone();
                                south_anchor.set_area((tempx, tempy));
                                mover.add_goal( Goal::new(1, south_anchor) );
                                tempy -= 1;
                            }
                        }

                        let mut dest_anchor = map.anchor_points[gx + gy * map.width].clone();
                        dest_anchor.set_area((ax, ay));

                        mover.add_goal( Goal::new(1, dest_anchor) );
                    }*/
                },
                _ => {},
            }
        }
    }
}