use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    renderer::SpriteRender,
};
use amethyst::ecs::prelude::{Entity, Entities};
use crate::game_state::{Map, SpriteSheetHandles, SpriteSheetLabel, 
    Config, Dimensions, KeyCheck, DEFAULT_BASE_SPEED, TILE_SIZE};
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
            println!("spawning for area");
            let mut c = 0;
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

                c += 1;
                //println!("{} spawning at {:?} {:?}", c, ((cux * TILE_SIZE + TILE_SIZE / 2) as f32, (cuy * TILE_SIZE + TILE_SIZE / 2) as f32), map.location);
                
                let mut local_transform = Transform::default();
                local_transform.set_translation_xyz(-100.0, 0.0, 0.0);

                let local_physical = Physical::new(((cux * TILE_SIZE + TILE_SIZE / 2) as f32, (cuy * TILE_SIZE + TILE_SIZE / 2) as f32), (ax, ay));
                let local_render = SpriteRender {
                    sprite_sheet: handles.get(SpriteSheetLabel::Person).unwrap().clone(),
                    sprite_number: 1,
                };
                let local_ids = Id::new();
                let local_mover = Mover::new(DEFAULT_BASE_SPEED);
                let local_off = Offscreen::new();

                ents.build_entity()
                    .with(local_transform, &mut trans)
                    .with(local_physical, &mut phys)
                    .with(local_render, &mut srs)
                    .with(local_ids, &mut ids)
                    .with(local_mover, &mut movers)
                    .with(local_off, &mut offs)
                    .build();
            }
            map.spawned = true;
        }
    }
}