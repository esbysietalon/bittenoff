use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
};
use crate::game_state::{Map, EntityType, GoalPriority, GoalType, BASE_OFFSCREEN_HUNGER_RELIEF_CHANCE};
use crate::components::{Hunger, Mover, Offscreen, Physical, Goal, Plant, Id};
use rand::Rng;

pub struct HungerSystem;

impl<'s> System<'s> for HungerSystem{
    type SystemData = (
        WriteStorage<'s, Hunger>,
        ReadStorage<'s, Offscreen>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut hungs, offs, time): Self::SystemData) {
        for hung in (&mut hungs).join() {
            if hung.get_hunger() > 0.0 {
                hung.mut_hunger(-1.0 * hung.get_rate() * time.delta_seconds());
            }
        }
        for (hung, off) in (&mut hungs, &offs).join() {
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() > off.time_passed() * BASE_OFFSCREEN_HUNGER_RELIEF_CHANCE {
                hung.mut_hunger(hung.get_capacity());
            }
        }
    }
}

pub struct GoalSystem;

impl<'s> System<'s> for GoalSystem{
    type SystemData = (
        WriteStorage<'s, Hunger>,
        WriteStorage<'s, Mover>,
        ReadStorage<'s, Physical>,
        ReadStorage<'s, Id>,
        ReadStorage<'s, Plant>,
        Read<'s, Time>,
        Read<'s, Map>,
    );

    fn run(&mut self, (mut hungs, mut movers, physicals, ids, plants, time, map): Self::SystemData) {
        for (hung, mover, phys) in (&mut hungs, &mut movers, &physicals).join() {
            
            if (hung.get_hunger() < hung.get_capacity() / 0.3) || (hung.get_hunger() < hung.get_rate() * 240.0) {
                //hungry: go find meal
                //println!("hungry!");
                //search for meal in local area
                if phys.get_location() != map.location {
                    continue;
                }
                if mover.has_goal_type(GoalType::MealGoal) {
                    continue;
                }


                let (tx, ty) = phys.get_tile_position();
                let x = tx as isize;
                let y = ty as isize;
                let mut meal_found = false;
                let mut range = 1;
                let upper_range = map.width.max(map.height) as isize;

                let mut mx = 0;
                let mut my = 0;
                while !meal_found && range < upper_range {
                    //println!("still searching at range {}", range);
                    for ix in (x-range)..(x+range+1) {
                        if ix < 0 || ix >= map.width as isize {
                            continue;
                        }
                        for iy in (y-range)..(y+range+1) {   
                            if iy < 0 || iy >= map.height as isize {
                                continue;
                            }
                            if ix as usize + iy as usize * map.width >= map.entities.len() {
                                continue;
                            }
                            //println!("valid tile");
                            if map.entities[ix as usize + iy as usize * map.width].get_type() == EntityType::Plant {
                                //println!("found plant");
                                for (id, plant) in (&ids, &plants).join() {
                                    if id.get_uuid() == map.entities[ix as usize + iy as usize * map.width].get_uuid() {
                                        if plant.get_fruit_progress() >= 1.0 {
                                            //println!("found meal");
                                            hung.set_meal_id(id.clone());
                                            meal_found = true;
                                            mx = ix as usize;
                                            my = iy as usize;
                                            break;
                                        }
                                    }
                                }
                                if meal_found {
                                    break;
                                }
                            }
                        }
                        if meal_found {
                            break;
                        }
                    }
                    range += 1;
                }
                
                let mut goal_added = false;

                if meal_found && mx + my * map.width + 4 < map.anchor_points.len() {
                    //println!("adding meal goal");
                    mover.add_goal(Goal::new(GoalPriority::MealGoal as usize, map.anchor_points[mx + my * map.width + 4].clone(), GoalType::MealGoal));
                    goal_added = true;
                }

                if mover.has_goal_type(GoalType::MealSearch) {
                    continue;
                }

                //if no meal found in local area, search for meal in other area
                if !goal_added {
                    //println!("searched for meal but did not find");
                    let mut rng = rand::thread_rng();
                        
                    let mut ax = rng.gen_range(-1, 2);
                    let mut ay = rng.gen_range(-1, 2);

                    if ax == 0 && ay == 0 {
                        ax = rng.gen_range(-1, 2);
                        ay = rng.gen_range(-1, 2);
                    }

                    let gx = rng.gen_range(0, map.width);
                    let gy = rng.gen_range(0, map.height);

                    let index = gx + gy * map.width;
                    //if (ax, ay) == map.location {
                    if index < map.width * map.height && index < map.anchor_points.len() {
                        //println!("adding goal {:?}", map.anchor_points[gx + gy * map.width].pos);
                        let mut dest_anchor = map.anchor_points[gx + gy * map.width + 4].clone();
                        dest_anchor.set_area((ax, ay));

                        //println!("destination anchor {:?}", dest_anchor);
                        
                        mover.add_goal( Goal::new(GoalPriority::MealSearch as usize, dest_anchor, GoalType::MealSearch) );
                    }
                }
            }
        }
    }
}