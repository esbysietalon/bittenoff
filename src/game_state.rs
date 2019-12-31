extern crate amethyst;

extern crate num;

use crate::components;

use std::thread;
use std::thread::JoinHandle;
use std::fs;

use rand::Rng;

use std::sync::{Arc};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use serde::Deserialize;
use ron::de::from_str;

use noise::{Seedable, NoiseFn, Perlin, Billow};

use amethyst::ecs::prelude::Entity;
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    core::timing::Time,
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};


use crate::components::Id;

pub const PLAYER_WIDTH: usize = 1;
pub const PLAYER_HEIGHT: usize = 1;
pub const PERSON_NUM: u32 = 25;

pub const TILE_SIZE: usize = 16;
pub const ENTITY_LIM: usize = 100;

pub const ZOOM_FACTOR: f64 = 0.5;
pub const ADJUSTMENT_ZOOM_FACTOR: f64 = 0.25;
pub const NOISE_DISPLACEMENT: f64 = 0.5;

pub const TICK_RATE: f32 = 0.5;

#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum Tile {
    GrassyHeavy = 0,
    Grassy,
    Plain,
    RockyLight,
    Rocky,
    Size
}

#[derive(Default, Clone)]
pub struct Area{
    pub tiles: Vec<Tile>,
    pub n: usize,
    pub e: usize,
    pub w: usize,
    pub s: usize,
    pub nw: usize,
    pub ne: usize,
    pub sw: usize,
    pub se: usize,
}

impl Area{
    pub fn new() -> Area{
        Area {
            tiles: Vec::new(),
            n: usize::max_value(),
            e: usize::max_value(),
            w: usize::max_value(),
            s: usize::max_value(),
            nw: usize::max_value(),
            ne: usize::max_value(),
            sw: usize::max_value(),
            se: usize::max_value(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Map{
    pub width: usize, 
    pub height: usize,
    pub world_seed: (f64, f64, f64, f64),
    pub location: (i32, i32),
    pub tiles: Vec<Tile>,
    pub entities: Vec<Id>,
    pub world_map: Vec<Area>,
    pub area_index: usize,
    pub rerolled: bool,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height, 
            world_seed: (0.0, 0.0, 0.0, 0.0),
            location: (0, 0),
            tiles: vec![Tile::Size; width * height],
            entities: vec![Id::nil(); ENTITY_LIM],
            world_map: Vec::new(),
            area_index: 0,
            rerolled: false,
        }   
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Config{
    pub stage_height: f32,
    pub stage_width: f32,
    pub spritesheet_name: String,
}

#[derive(Default)]
pub struct LoadingState{
    pub config_path: String,
    pub loading: Arc<AtomicBool>,
    pub load_thread: Option<JoinHandle<(Config)>>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>
}

#[derive(Default)]
pub struct PlayState{
    //sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

pub fn update_world_seed(map: &mut Map, dir: char){
    match dir {
        'n' => {
            map.world_seed.1 += 1.0 / ZOOM_FACTOR;
            map.world_seed.3 += 1.0 / ADJUSTMENT_ZOOM_FACTOR;
            map.location.1 += 1;
        }
        'w' => {
            map.world_seed.0 -= 1.0 / ZOOM_FACTOR;
            map.world_seed.2 -= 1.0 / ADJUSTMENT_ZOOM_FACTOR;
            map.location.0 -= 1;
        }
        'e' => {
            map.world_seed.0 += 1.0 / ZOOM_FACTOR;
            map.world_seed.2 += 1.0 / ADJUSTMENT_ZOOM_FACTOR;
            map.location.0 += 1;
        }
        's' => {
            map.world_seed.1 -= 1.0 / ZOOM_FACTOR;
            map.world_seed.3 -= 1.0 / ADJUSTMENT_ZOOM_FACTOR;
            map.location.1 -= 1;
        }
        _ => {}
    }
    println!("area is {:?}", map.location);
}

pub fn load_map(map: &mut Map, to_load: (Option<Area>, usize)) {
    println!("loading area to map from {}", to_load.1);

    let mut area_pointer = &(Area::new());
    match to_load.0 {
        Some(a) => {
            map.world_map.push(a);
            area_pointer = &map.world_map[to_load.1];
        }
        None => {
            area_pointer = &map.world_map[to_load.1];
        }
    }

    for i in 0..map.tiles.len() {
        map.tiles[i] = (*area_pointer).tiles[i];
    }

    map.area_index = to_load.1;
    map.rerolled = true;
}

pub fn regenerate_map(map: &mut Map, area_index: usize, direction: char) -> (Option<Area>, usize){
    let old_map_len = map.world_map.len();

    let mut new_area_index = &mut (Area::new()).n;

    let mut diagonal_a = usize::max_value(); //n/w lean
    let mut diagonal_b = usize::max_value(); //s/e lean

    let mut diagonal_c = usize::max_value(); //n/w lean
    let mut diagonal_d = usize::max_value(); //s/e lean

    {
        if direction == 'e' {
            diagonal_a = map.world_map[area_index].n;
            diagonal_b = map.world_map[area_index].s;

            diagonal_c = map.world_map[area_index].ne;
            diagonal_d = map.world_map[area_index].se;

            new_area_index = &mut ((map.world_map[area_index]).e);
        }else if direction == 'w' {
            diagonal_a = map.world_map[area_index].n;
            diagonal_b = map.world_map[area_index].s;
            
            diagonal_c = map.world_map[area_index].nw;
            diagonal_d = map.world_map[area_index].sw;

            new_area_index = &mut ((map.world_map[area_index]).w);
        }else if direction == 's' {
            diagonal_a = map.world_map[area_index].w;
            diagonal_b = map.world_map[area_index].e;

            diagonal_c = map.world_map[area_index].sw;
            diagonal_d = map.world_map[area_index].se;

            new_area_index = &mut ((map.world_map[area_index]).s);
        }else if direction == 'n' {
            diagonal_a = map.world_map[area_index].w;
            diagonal_b = map.world_map[area_index].e;
            
            diagonal_c = map.world_map[area_index].nw;
            diagonal_d = map.world_map[area_index].ne;
            
            new_area_index = &mut ((map.world_map[area_index]).n);
        }   
    }
    
    println!("area lookup is {} (throwaway value is {})", *new_area_index, usize::max_value());


    if *new_area_index == usize::max_value() {
        println!("creating new area");

        let mut area = Area::new();

        let mut rng = rand::thread_rng();

        let w = map.width;
        let h = map.height;

        let perlin = Perlin::new();
        let billow = Billow::new();

        //perlin.set_seed(rng.gen::<u32>());

        let xseed = map.world_seed.0;
        let yseed = map.world_seed.1;
        let aseed = map.world_seed.2;
        let bseed = map.world_seed.3;


        for y in 0..h {
            for x in 0..w {
                let noise = perlin.get([xseed + x as f64 / w as f64 / ZOOM_FACTOR, yseed + y as f64 / h as f64 / ZOOM_FACTOR]);
                let adjustment = billow.get([aseed + x as f64 / w as f64 / ADJUSTMENT_ZOOM_FACTOR, bseed + y as f64 / h as f64 / ADJUSTMENT_ZOOM_FACTOR]) * NOISE_DISPLACEMENT;
                //(rng.gen::<f64>() - rng.gen::<f64>()) * NOISE_DISPLACEMENT;//
                let mut tilefloat = noise_ease(noise + adjustment);
                
                if tilefloat >= 1.0 {
                    tilefloat = 0.99;
                }else if tilefloat < 0.0 {
                    tilefloat = 0.0;
                }

                let tile = num::FromPrimitive::from_u32((tilefloat * (Tile::Size as i32 as f64)) as u32).unwrap();
                area.tiles.push(tile);
                map.tiles[x + y * w] = area.tiles[x + y * w];
            }
        }

        //link the list backwards
        if direction == 'n' {
            area.sw = diagonal_a;
            area.se = diagonal_b;
            area.w = diagonal_c;
            area.e = diagonal_d;
            area.s = map.area_index;
        }else if direction == 'e' {
            area.nw = diagonal_a;
            area.sw = diagonal_b;
            area.n = diagonal_c;
            area.s = diagonal_d;
            area.w = map.area_index;
        }else if direction == 'w' {
            area.ne = diagonal_a;
            area.se = diagonal_b;
            area.n = diagonal_c;
            area.s = diagonal_d;
            area.e = map.area_index;
        }else if direction == 's' {
            area.nw = diagonal_a;
            area.ne = diagonal_b;
            area.w = diagonal_c;
            area.e = diagonal_d;
            area.n = map.area_index;
        }

        //link the list forwards
        *new_area_index = old_map_len;
        //map.world_map.push(area);
        (Some(area), *new_area_index)
    }else{
        println!("no new area needed; found area {}", *new_area_index);

        //let area = &map.world_map[*new_area_index];

        //for i in 0..map.tiles.len() {
        //    map.tiles[i] = (*area).tiles[i];
        //}
        (None, *new_area_index)
    }
}


fn noise_ease(raw: f64) -> f64{
    let abs = raw.abs();

    abs
}

fn generate_map(world: &mut World){
    let mut map = world.write_resource::<Map>();

    let mut area = Area::new();

    let mut rng = rand::thread_rng();

    let w = map.width;
    let h = map.height;

    let perlin = Perlin::new();

    let billow = Billow::new();

    //perlin.set_seed(rng.gen::<u32>());

    let xseed = map.world_seed.0;
    let yseed = map.world_seed.1;
    let aseed = map.world_seed.2;
    let bseed = map.world_seed.3;

    for y in 0..h {
        for x in 0..w {
            let noise = perlin.get([xseed + x as f64 / w as f64 / ZOOM_FACTOR, yseed + y as f64 / h as f64 / ZOOM_FACTOR]);
            let adjustment = billow.get([aseed + x as f64 / w as f64 / ADJUSTMENT_ZOOM_FACTOR, bseed + y as f64 / h as f64 / ADJUSTMENT_ZOOM_FACTOR]) * NOISE_DISPLACEMENT;//(rng.gen::<f64>() - rng.gen::<f64>()) * NOISE_DISPLACEMENT;

            //println!("adjustment {}", adjustment);

            let mut tilefloat = noise_ease(noise + adjustment);
            
            if tilefloat >= 1.0 {
                tilefloat = 0.99;
            }else if tilefloat < 0.0 {
                tilefloat = 0.0;
            }

            //println!("tilefloat {}", tilefloat);

            let tile = num::FromPrimitive::from_u32((tilefloat * (Tile::Size as i32 as f64)) as u32).unwrap();
            //println!("tile {:?}", tile);
            area.tiles.push(tile);
            map.tiles[x + y * w] = area.tiles[x + y * w];
        }
    }

    /*
    for i in 0..(map.width * map.height) {
        area.tiles.push(num::FromPrimitive::from_u32(rng.gen_range(0, Tile::Size as u32)).unwrap());
        map.tiles[i] = area.tiles[i];
    }*/

    map.world_map.push(area);
}

fn initialise_tiles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let s_w = world.read_resource::<Config>().stage_width;
    let s_h = world.read_resource::<Config>().stage_height;

    let tile_num_w = s_w as usize / TILE_SIZE + 1;
    let tile_num_h = s_h as usize / TILE_SIZE + 1;


    let mut tiles = Vec::new();
    
    {
        let map = &world.read_resource::<Map>();
        tiles = map.tiles.clone();
    }

    for y in 0..tile_num_h {
        for x in 0..tile_num_w {
            let mut local_transform = Transform::default();

            let tile = tiles[x + y * tile_num_w];
            
            local_transform.set_translation_xyz((x * TILE_SIZE) as f32, (y * TILE_SIZE) as f32, 0.0);
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: tile as usize,
            };

            world
                .create_entity()
                .with(sprite_render)
                .with(local_transform)
                .with(components::Tile::new((x + y * tile_num_w) as usize))
                .build();
        }
    }

}
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    
    let s_w = world.read_resource::<Config>().stage_width;
    let s_h = world.read_resource::<Config>().stage_height;

    transform.set_translation_xyz(s_w * 0.5, s_h * 0.5, 2.0);

    world
        .create_entity()
        .with(Camera::standard_2d(s_w, s_h))
        .with(transform)
        .build();
}
fn initialise_player(world: &mut World, sprite_sheet: Handle<SpriteSheet>){
    let mut local_transform = Transform::default();
    
    let s_w = world.read_resource::<Config>().stage_width;
    let s_h = world.read_resource::<Config>().stage_height;

    local_transform.set_translation_xyz(-100.0, 0.0, 0.0);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet,
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(components::Player {
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
        })
        .with(components::Id::new())
        .with(components::Physical::new((s_w / 2.0, s_h / 2.0), (0, 0)))
        .with(local_transform)
        .build();
}
fn initialise_persons(world: &mut World, sprite_sheet: Handle<SpriteSheet>){
    for _ in 0..PERSON_NUM{
        let mut local_transform = Transform::default();
        
        let s_w = world.read_resource::<Config>().stage_width;
        let s_h = world.read_resource::<Config>().stage_height;

        let mut rng = rand::thread_rng();

        local_transform.set_translation_xyz(-100.0, 0.0, 0.0);
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 1,
        };

        world
            .create_entity()
            .with(sprite_render)
            .with(components::Id::new())
            .with(components::Physical::new((rng.gen_range(0 as u32, s_w as u32) as f32, rng.gen_range(0 as u32, s_h as u32) as f32), (rng.gen_range(-1, 2), rng.gen_range(-1, 2))))
            .with(local_transform)
            .build();
    }
}

fn load_sprite_sheet(world: &mut World, name: &str) -> Handle<SpriteSheet> {
    //loading spritesheet
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            format!("{}{}{}", "res/textures/",name,".png"),
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };
    
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}{}{}", "res/textures/",name,".ron"),
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}


impl SimpleState for LoadingState {
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>){
        self.loading = Arc::new(AtomicBool::new(false));
        let load = self.loading.clone();
        let path = self.config_path.clone(); 
        self.load_thread.replace(thread::spawn(move || {        
            if !(*load).load(Ordering::Relaxed) {
                println!("Starting load thread!");
                let contents = fs::read_to_string(path)
                    .expect("Error reading config file");
                let loaded: Config = from_str(&contents)
                    .expect("Error loading config file");
                (*load).store(true, Ordering::Relaxed);
                println!("Loaded!");
                loaded
            }else{
                Config::default()
            }
        }));

        println!("Started loading");
    }
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>> ) -> SimpleTrans{
        if (*self.loading).load(Ordering::Relaxed) {
            let loaded = self.load_thread.take().unwrap().join().expect("Error encountered while joining thread");
            
            //NOTICE Map is defined here
            let mut map = Map::new(loaded.stage_width as usize / TILE_SIZE + 1, loaded.stage_height as usize / TILE_SIZE + 1);
            
            //seeding map world seed
            let mut rng = rand::thread_rng();
            map.world_seed = (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>());

            println!("Loaded config: {:?}", loaded);
            data.world.insert(loaded);
            data.world.insert(map);


            let world = &mut data.world;

            generate_map(*world);

            self.sprite_sheet_handle.replace(load_sprite_sheet(*world, "tiles"));
    
            initialise_tiles(*world, self.sprite_sheet_handle.clone().unwrap());
            
            self.sprite_sheet_handle.replace(load_sprite_sheet(*world, "player"));
            
            initialise_player(*world, self.sprite_sheet_handle.clone().unwrap());
    
            initialise_persons(world, self.sprite_sheet_handle.clone().unwrap());
            
            initialise_camera(*world);

            data.world
                .create_entity()
                .with(components::Counter::new())
                .build();

            Trans::Switch(Box::new(PlayState::default()))
        }else{
            println!("Loading..");
            Trans::None
        }
    }
    fn on_stop(&mut self, _data: StateData<'_, GameData<'_, '_>>){
        println!("Stopped loading");
    }
}

impl SimpleState for PlayState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>){
        println!("Entering play state..");
        
    }
}