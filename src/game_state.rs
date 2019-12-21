extern crate amethyst;

use crate::components;

use std::thread;
use std::thread::JoinHandle;
use std::fs;

use std::sync::{Arc};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use serde::Deserialize;
use ron::de::from_str;

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
pub const PERSON_NUM: u32 = 15;
pub const TILE_SIZE: usize = 16;
pub const TICK_RATE: f32 = 0.5;

#[derive(Clone, Copy)]
pub enum Tile {
    Null = 0, 
    Grass,
}

#[derive(Default, Clone)]
pub struct Map{
    pub width: usize, 
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub entities: Vec<Id>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![Tile::Null; width * height],
            entities: vec![Id::nil(); width * height],
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
}

#[derive(Default)]
pub struct PlayState{
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}


fn initialise_tiles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let s_w = world.read_resource::<Config>().stage_width;
    let s_h = world.read_resource::<Config>().stage_height;

    let tile_num_w = s_w as usize / TILE_SIZE + 1;
    let tile_num_h = s_h as usize / TILE_SIZE + 1;

    for y in 0..tile_num_h {
        for x in 0..tile_num_w {
            let mut local_transform = Transform::default();

            {
                let mut map = world.write_resource::<Map>();
            }

            let tile = Tile::Grass;

            local_transform.set_translation_xyz((x * TILE_SIZE) as f32, (y * TILE_SIZE) as f32, 0.0);
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: tile as usize,
            };

            world
                .create_entity()
                .with(sprite_render)
                .with(local_transform)
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

    local_transform.set_translation_xyz(s_w / 2.0, s_h / 2.0, 0.0);
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
        .with(components::Physical::new(components::Physical::into_tile_position((local_transform.translation().x, local_transform.translation().y))))
        .with(local_transform)
        .build();
}
fn initialise_persons(world: &mut World, sprite_sheet: Handle<SpriteSheet>){
    for _ in 0..PERSON_NUM{
        let mut local_transform = Transform::default();
        
        let s_w = world.read_resource::<Config>().stage_width;
        let s_h = world.read_resource::<Config>().stage_height;

        local_transform.set_translation_xyz(s_w / 2.0, s_h / 2.0, 0.0);
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 1,
        };

        world
            .create_entity()
            .with(sprite_render)
            .with(components::Id::new())
            .with(components::Physical::new(components::Physical::into_tile_position((local_transform.translation().x, local_transform.translation().y))))
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
            let map = Map::new(loaded.stage_width as usize, loaded.stage_height as usize);
            
            
            println!("Loaded config: {:?}", loaded);
            data.world.insert(loaded);
            data.world.insert(map);

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
        let world = data.world;

        self.sprite_sheet_handle.replace(load_sprite_sheet(world, "tiles"));

        initialise_tiles(world, self.sprite_sheet_handle.clone().unwrap());
        
        self.sprite_sheet_handle.replace(load_sprite_sheet(world, "spritesheet"));
        
        initialise_player(world, self.sprite_sheet_handle.clone().unwrap());

        //initialise_persons(world, self.sprite_sheet_handle.clone().unwrap());
        initialise_camera(world);
    }
}