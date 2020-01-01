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
            match mover.pop_move() {
                Some(a) => {
                    //println!("next move {:?}", a);
                    let (x, y) = a;
                    phys.set_x(x);
                    phys.set_y(y);
                }
                None => {
                    println!("change goal, no moves detected");
                    mover.pop_goal();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Pos(i32, i32);

impl Pos {
  fn distance(&self, other: &Pos) -> u32 {
    (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as u32
  }

  fn successors(&self, range: i32, w: i32, h: i32) -> Vec<(Pos, u32)> {
    let &Pos(x, y) = self;
    let mut succ = Vec::new();
    for ny in -range..(range + 1) {
        if y + ny < 0 || y + ny >= h {
            continue;
        }
        for nx in -range..(range + 1) {
            if x + nx < 0 || x + nx >= w || (ny == 0 && nx == 0) {
                continue
            }
            succ.push((Pos(x + nx, y + ny), 1));
        }
    }
    succ
  }
}

pub struct RudderSystem;

impl<'s> System<'s> for RudderSystem{
    type SystemData = (
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Mover>,
        Read<'s, Config>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut physicals, mut movers, config, time): Self::SystemData) {
        for (mover, phys) in (&mut movers, &mut physicals).join(){
            if !mover.is_move_vec_empty() {
                continue;
            }
            match mover.pop_goal() {
                Some((gx, gy)) => {
                    //println!("goal is {:?}", (gx, gy));

                    let move_range_increase = 2.0;//mover.speed() * time.delta_seconds();
                    mover.inc_range(move_range_increase);
                    
                    let mut range = mover.range() as i32;

                    if range > 0 {
                        //subtract truncated range from move_range
                        mover.inc_range(-range as f32);

                        let goal = Pos(gx as i32, gy as i32);

                        //println!("goal is {:?}", goal);

                        let (ox, oy) = phys.get_real_position();

                        //println!("origin is {:?}", (ox, oy));

                        let calc_path =  astar(&Pos(ox as i32, oy as i32), |p| p.successors(range, config.stage_width as i32, config.stage_height as i32), |p| p.distance(&goal) / 3,
                        |p| *p == goal);

                        match calc_path {
                            Some((v, c)) => {
                                //println!("path is {:?} at {}", v, c);
                                let path: Vec<(f32, f32)> = v.into_iter().map(|Pos(a, b)| {
                                    (a as f32, b as f32)
                                }).collect();
                                
                                //path.reverse();
                                
                                mover.set_move_vec(path);
                            }
                            None => {}
                        }
                    }
                }
                None => {}
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
                    
                    let (x, y) = phys.get_real_position();
                    let dx = rng.gen::<f32>() * 100.0 - rng.gen::<f32>() * 100.0;
                    let dy = rng.gen::<f32>() * 100.0 - rng.gen::<f32>() * 100.0;
                    //println!("adding goal");
                    let gx = x + dx;
                    let gy = y + dy;
                    
                    if gx >= 0.0 && gx < config.stage_width && gy >= 0.0 && gy < config.stage_height {
                        mover.add_goal(Goal::new(1, (gx, gy)));
                    }
                },
                _ => {},
            }
        }
    }
}