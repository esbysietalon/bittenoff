use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    renderer::SpriteRender,
};
use amethyst::ecs::prelude::{Entity, Entities};
use crate::game_state::{Map, SpriteSheetHandles, SpriteSheetLabel, 
    Config, Dimensions, KeyCheck, DEFAULT_BASE_SPEED, TILE_SIZE,
    PLANT_NUM_LOWER, PLANT_NUM_UPPER,
    spawn_person, spawn_plant};
use crate::components::{Tile, Mover, Id, Physical, Offscreen, Hunger, Plant};

use rand::Rng;

pub struct MapSystem;

impl<'s> System<'s> for MapSystem{
    type SystemData = (
        Write<'s, Map>,
        WriteStorage<'s, Tile>,
        WriteStorage<'s, SpriteRender>,
    );

    fn run(&mut self, (mut map, mut tiles, mut sprite_renders): Self::SystemData) {
        if map.rerolled {
            for (tile, sprite_render) in (&mut tiles, &mut sprite_renders).join(){
                sprite_render.sprite_number = map.tiles[tile.index()].tile as usize;
            }
            map.rerolled = false;
        }
    }
}


pub struct SpawnSystem; 

impl<'s> System<'s> for SpawnSystem{
    type SystemData = (
        Write<'s, Map>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Mover>,
        WriteStorage<'s, Offscreen>,
        WriteStorage<'s, Physical>,
        WriteStorage<'s, Hunger>,
        WriteStorage<'s, Id>,
        WriteStorage<'s, Plant>,
        Entities<'s>,
        Read<'s, SpriteSheetHandles>,
    );

    fn run(&mut self, (mut map, mut trans, mut srs, mut movers, mut offs, mut phys, mut hungs, mut ids, mut plants, mut ents, handles): Self::SystemData) {
        if !map.spawned {
            //spawning plants
            let mut rng = rand::thread_rng();
            let plant_num = rng.gen_range(PLANT_NUM_LOWER, PLANT_NUM_UPPER);
            //let plant_num = 15;

            for _i in 0..plant_num {
                let cux = 10;
                let cuy = 10;
                //let mut cux = rng.gen_range(0, map.width);
                //let mut cuy = rng.gen_range(0, map.height);

                let mut fail_find = false;
                let mut try_counter = 0;
                /*while !map.is_passable((cux, cuy)) {
                    cux = rng.gen_range(0, map.width);
                    cuy = rng.gen_range(0, map.height);
                    try_counter += 1;
                    if try_counter > 5 {
                        fail_find = true;
                        break;
                    }
                }*/

                if fail_find {
                    continue;
                }
                

                spawn_plant(cux, cuy, map.location.0, map.location.1, &handles, &mut ents, &mut phys, &mut plants, &mut ids, &mut offs, &mut trans, &mut srs);
            }
            //spawning persons
            for rect in map.structures.clone() {
                let mut rng = rand::thread_rng();

                let mut adx = rng.gen_range(-1, 2);
                let mut ady = rng.gen_range(-1, 2);
                
                if rng.gen::<f32>() > 0.2 {
                    adx = 0;
                    ady = 0;
                }

                let mut cux = rng.gen_range(0, map.width);
                let mut cuy = rng.gen_range(0, map.height);

                if adx == 0 && ady == 0 {
                    let mut fail_find = false;
                    let mut try_counter = 0;
                    while !map.is_passable((cux, cuy)) {
                        cux = rng.gen_range(0, map.width);
                        cuy = rng.gen_range(0, map.height);
                        try_counter += 1;
                        if try_counter > 10 {
                            fail_find = true;
                            break;
                        }
                    }

                    if fail_find {
                        continue;
                    }
                }
                
                let ax = map.location.0 + adx;
                let ay = map.location.1 + ady;

                spawn_person(cux, cuy, ax, ay, &handles, &mut ents, &mut phys, &mut movers, &mut ids, &mut offs, &mut trans, &mut srs, &mut hungs);                
            }
            map.spawned = true;
        }
    }
}