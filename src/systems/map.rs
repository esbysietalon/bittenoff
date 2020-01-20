use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    renderer::SpriteRender,
};
use amethyst::ecs::prelude::{Entity, Entities};
use crate::game_state::{Map, SpriteSheetHandles, SpriteSheetLabel, 
    Config, Dimensions, KeyCheck, DEFAULT_BASE_SPEED, TILE_SIZE,
    spawn_person};
use crate::components::{Tile, Mover, Id, Physical, Offscreen};

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
        WriteStorage<'s, Id>,
        Entities<'s>,
        Read<'s, SpriteSheetHandles>,
    );

    fn run(&mut self, (mut map, mut trans, mut srs, mut movers, mut offs, mut phys, mut ids, mut ents, handles): Self::SystemData) {
        if !map.spawned {
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
                    while !map.is_passable((cux, cuy)) {
                        cux = rng.gen_range(0, map.width);
                        cuy = rng.gen_range(0, map.height);
                    }
                }
                
                let ax = map.location.0 + adx;
                let ay = map.location.1 + ady;

                spawn_person(cux, cuy, ax, ay, &handles, &mut ents, &mut phys, &mut movers, &mut ids, &mut offs, &mut trans, &mut srs);                
            }
            map.spawned = true;
        }
    }
}