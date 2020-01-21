extern crate amethyst;

extern crate num;

use crate::components;

use std::thread;
use std::thread::JoinHandle;
use std::fs;

use std::f32::consts::PI;

use rand::Rng;

use std::sync::{Arc};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use serde::Deserialize;
use ron::de::from_str;

use noise::{Seedable, NoiseFn, Perlin, Billow};

use amethyst::ecs::prelude::{Read, Entity, Entities, WriteStorage};
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    core::timing::Time,
    core::math::Vector3,
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};


use crate::components::{Id, Particle, ParticleDeathType};

pub const PLAYER_WIDTH: usize = 1;
pub const PLAYER_HEIGHT: usize = 1;
pub const PLAYER_SPEED: f32 = 120.0;

pub const PERSON_NUM: u32 = 25;

pub const DEFAULT_HUNGER_RATE: f32 = 1.0; 
pub const DEFAULT_HUNGER_CAPACITY: f32 = 60.0 * 10.0; //in seconds / DEFAULT_HUNGER_RATE till starving

pub const PLANT_NUM_LOWER: usize = 15;
pub const PLANT_NUM_UPPER: usize = 20;

pub const DEFAULT_BASE_SPEED: f32 = 120.0;

pub const TILE_SIZE: usize = 16;
pub const ENTITY_LIM: usize = 50;

pub const ZOOM_FACTOR: f64 = 0.5;
pub const ADJUSTMENT_ZOOM_FACTOR: f64 = 0.25;
pub const NOISE_DISPLACEMENT: f64 = 0.5;

pub const OFFSCREEN_UNKNOWN_PATH_WAIT_TIME: f32 = 10.0 * 100.0 / DEFAULT_BASE_SPEED; //in seconds

pub const STRUCTURE_RESOLUTION_FACTOR: f32 = 0.25;
pub const BIOME_RESOLUTION_FACTOR: f32 = 0.1;

pub const BIOME_NUM: u32 = 2;
pub const BIOME_TILESET_SIZE: u32 = 5;

pub const MAP_SEED_RANGE: i32 = i32::max_value();

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum KeyCheck {
    Enter = 0,
    E,
    Size,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum SpellComponent {
    Force,
    Fire,
    With,
    Upon,
    Impact,
    Size,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum EntityType {
    Person,
    Plant,
    Size,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Plant {
    BushBerryRipe = 0,
    BushBerryNoBerry,
}

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub key_check: Vec<bool>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            key_check: vec![false; KeyCheck::Size as usize],
        }
    }
}

#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum Tile {
    Plain = 0,
    Grassy,
    RockyLight,
    GrassyHeavy,
    Rocky,
    Sandy,
    SandySparse,
    SandyRocky,
    SandyWeed,
    SandyBoulder,
    WoodFloor,
    WoodWall,
    Size,
}

#[derive(Clone, Copy, Debug)]
pub struct TileBlock {
    pub tile: Tile,
    pub passable: bool,
}

impl TileBlock {
    pub fn new(tile: Tile, passable: bool) -> TileBlock {
        TileBlock {
            tile,
            passable,
        }
    }
}

#[derive(Eq, Debug, Hash)]
pub struct Anchor{
    pub pos: (usize, usize, i32, i32),
    pub succ: Vec<(usize, usize)>,
}

impl Anchor {
    pub fn new(x: usize, y: usize, lx: i32, ly: i32) -> Anchor {
        Anchor {
            pos: (x, y, lx, ly),
            succ: Vec::new(),
        }
    }
    pub fn real_local(&self) -> (f32, f32) {
        let (x, y, _, _) = self.pos;
        let mut nx = (TILE_SIZE / 2 + x * TILE_SIZE) as f32;
        if x == usize::max_value() {
            nx = (-(TILE_SIZE as i32) / 2) as f32; 
        }
        let mut ny = (TILE_SIZE / 2 + y * TILE_SIZE) as f32;
        if y == usize::max_value() {
            ny = (-(TILE_SIZE as i32) / 2) as f32; 
        }
        (nx, ny)
    }
    pub fn local(&self) -> (usize, usize) {
        (self.pos.0, self.pos.1)
    }
    pub fn area(&self) -> (i32, i32) {
        (self.pos.2, self.pos.3)
    }
    pub fn set_area(&mut self, new_area: (i32, i32)) {
        self.pos.2 = new_area.0;
        self.pos.3 = new_area.1;
    }
}

impl Clone for Anchor {
    fn clone(&self) -> Anchor {
        Anchor {
            pos: self.pos,
            succ: self.succ.to_vec(),
        }
    }
}

impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

#[derive(Default, Clone)]
pub struct Area{
    pub tiles: Vec<TileBlock>,
    pub anchor_points: Vec<Anchor>,
    pub structures: Vec<Rect>,
    pub spawned: bool,
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
            anchor_points: Vec::new(),
            structures: Vec::new(),
            spawned: false,
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
    pub world_seed: (f64, f64, f64, f64, f64, f64, f64, f64), // 0-4 tile-wise noise, 5-6 biome area-wise noise, 
    pub location: (i32, i32),
    pub tiles: Vec<TileBlock>,
    pub anchor_points: Vec<Anchor>,
    pub structures: Vec<Rect>,
    pub spawned: bool,
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
            world_seed: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            location: (0, 0),
            tiles: vec![TileBlock::new(Tile::Size, true); width * height],
            anchor_points: Vec::new(),
            structures: Vec::new(),
            spawned: false,
            entities: vec![Id::nil(); ENTITY_LIM],
            world_map: Vec::new(),
            area_index: 0,
            rerolled: false,
        }   
    }
    pub fn is_passable(&self, tile: (usize, usize)) -> bool {
        let index = tile.0 + tile.1 * self.width;
        if index < self.width * self.height {
            self.tiles[tile.0 + tile.1 * self.width].passable
        }else{
            true
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SpriteSheetLabel {
    Particles,
    Person,
    Plants,
    Tiles,
    UiTiles,
    Size,
}

impl Default for SpriteSheetLabel {
    fn default() -> Self {
        SpriteSheetLabel::Size
    }
}

#[derive(Default, Debug)]
pub struct SpriteSheetHandles{
    handles: Vec<(SpriteSheetLabel, Handle<SpriteSheet>)>,
}

impl SpriteSheetHandles {
    pub fn new() -> Self {
        SpriteSheetHandles {
            handles: Vec::new(),
        }
    }
    pub fn add(&mut self, label: SpriteSheetLabel, handle: Handle<SpriteSheet>) {
        println!("adding ({:?}, {:?})", label, handle);
        self.handles.push((label, handle));
    }
    pub fn get(&self, label: SpriteSheetLabel) -> Option<Handle<SpriteSheet>> {
        let mut out = None;
        println!("self.handles {:?}", self.handles);
        for (tag, handle) in self.handles.iter() {
            println!("tag {:?} handle {:?}", tag, handle);
            if *tag == label {
                out = Some(handle.clone());
                break;
            }
        }
        out
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
//where the Uis are defined
pub enum Ui {
    RuneBoard = 0,
    RuneDisplay,
    SpellDisplay,
    Size,
}

impl Default for Ui {
    fn default() -> Self {
        Ui::Size
    }
}

#[derive(Clone, Debug, Default)]
pub struct UiHolder {
    pub is_active: Vec<(Ui, bool)>,
}

impl UiHolder {
    pub fn new() -> UiHolder {
        UiHolder {
            is_active: Vec::new(),
        }
    }
    pub fn set_active(&mut self, index: usize, act: bool) {
        if index >= 0 && index < self.len() {
            self.is_active[index].1 = act;
        }
    }
    pub fn is_active(&self, index: usize) -> bool {
        if index >= 0 && index < self.len() {
            self.is_active[index].1
        }else{
            false
        }
    }
    pub fn is_type_active(&self, ui_type: Ui) -> bool {
        let mut out = false;
        for (ui, act) in self.is_active.iter() {
            if *ui == ui_type {
                out = *act;
                break;
            }
        }
        out
    }
    pub fn get_type(&self, index: usize) -> Ui {
        if index >= 0 && index < self.len() {
            (self.is_active[index]).0
        }else{
            Ui::default()
        }
    }
    pub fn len(&self) -> usize {
        self.is_active.len()
    }
    pub fn add_ui(&mut self, ui: Ui) -> usize {
        self.is_active.push((ui, false));
        self.len() - 1
    }
}

//dimensions of the screen
#[derive(Clone, Copy, Debug, Default)]
pub struct Dimensions{
    pub width: f32,
    pub height: f32,
}

impl Dimensions {
    pub fn new() -> Self {
        Dimensions {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn new((x, y): (usize, usize), (w, h): (usize, usize)) -> Self {
        Rect {
            x,
            y,
            w,
            h,
        }
    }
    pub fn is_in(&self, point: (usize, usize)) -> bool {
        (point.0 >= self.x && point.0 <= self.x + self.w && point.1 >= self.y && point.1 <= self.y + self.h)
    }
    pub fn check_collision(&self, o: &Rect) -> bool {
        self.is_in((o.x, o.y)) || o.is_in((self.x, self.y))
    }
    pub fn split(&self, line_x: usize, line_y: usize) -> Vec<Rect> {
        let mut out = Vec::new();
        if line_x > self.x && line_x < self.x + self.w {
            out.push(Rect::new((self.x, self.y), (line_x - self.x, self.h)));
            out.push(Rect::new((line_x, self.y), (self.x + self.w - line_x, self.h)));
        }
        if line_y > self.y && line_y < self.y + self.h {
            if out.len() > 0 {
                let mut temp = Vec::new();
                for rect in out.clone() {
                    temp.append(&mut rect.split(0, line_y));
                }
                out.clear();
                out.append(&mut temp);
            }else{
                out.push(Rect::new((self.x, self.y), (self.w, line_y - self.y)));
                out.push(Rect::new((self.x, line_y), (self.w, self.y + self.h - line_y)));
            }
        }
        out
    }
    
    pub fn shrink(&mut self, dist_x: usize, dist_y: usize) {
        //println!("shrinking from {:?}", self);
        let max_shrink_w = ((self.w / 2) as isize - 2).max(0) as usize;
        let max_shrink_h = ((self.h / 2) as isize - 2).max(0) as usize;
        if dist_x < max_shrink_w {
            self.x += dist_x;
            self.w -= dist_x * 2;
        }else{
            self.x += max_shrink_w;
            self.w -= max_shrink_w * 2;
        }
        if dist_y < max_shrink_h {
            self.y += dist_y;
            self.h -= dist_y * 2;
        }else{
            self.y += max_shrink_h;
            self.h -= max_shrink_h * 2;
        }
        //println!("shrinking to {:?}", self);
    }

    pub fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Config{
    pub stage_height: f32,
    pub stage_width: f32,
    pub spritesheet_name: String,
    pub fullscreen: bool,
    pub fps_limit: u32,
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
    //println!("area is {:?}", map.location);
}

pub fn generate_structures(area: &mut Area, location: (i32, i32), seed: (f64, f64), dim: (usize, usize)) {

    //println!("calling generate_structures");

    let (map_x, map_y) = location;
    let (seed_x, seed_y) = seed;
    let (map_width, map_height) = dim;
    let billow = Billow::new();
    let noise_check = billow.get([map_x as f64 * STRUCTURE_RESOLUTION_FACTOR as f64 + seed_x, map_y as f64 * STRUCTURE_RESOLUTION_FACTOR as f64 + seed_y]);

    let mut rect_squad = Vec::<Rect>::new();
    
    rect_squad.push(Rect::new((1, 1), (map_width-1, map_height-1)));

    let mut structure_num = ((noise_check.abs() - 0.4).max(0.0) / 0.1) as usize;

    //println!("setup complete; structure_num {}", structure_num);

    while rect_squad.len() < structure_num {
        let mut temp = Vec::new();

        let mut rng = rand::thread_rng();
        let randi = rng.gen_range(0, rect_squad.len());

        //println!("randi is {}", randi);

        let chosen_rect = rect_squad[randi];

        //println!("selecting rectangle {:?}", chosen_rect);

        if chosen_rect.w < 5 || chosen_rect.h < 5 {
            continue;
        }else{
            let dir = rng.gen_range(0, 2);
            let mut split_x = 0;
            let mut split_y = 0;

            if dir == 0 {
                //horizontal split
                let mut lb = chosen_rect.x + chosen_rect.w / 3;
                let mut rb = chosen_rect.x + chosen_rect.w * 2 / 3;
                if chosen_rect.w / 3 < 5 && chosen_rect.w / 2 >= 5 {
                    lb = chosen_rect.x + chosen_rect.w / 2;
                    rb = chosen_rect.x + chosen_rect.w / 2 + 1;
                }else if chosen_rect.w / 2 < 5{
                    structure_num -= 1;
                    continue;
                }
                //println!("left bound {} right bound {}", lb, rb);

                split_x = rng.gen_range(lb, rb);
            }else{
                //vertical split
                let mut lb = chosen_rect.y + chosen_rect.h / 3;
                let mut ub = chosen_rect.y + chosen_rect.h * 2 / 3;
                if chosen_rect.h / 3 < 5 && chosen_rect.h / 2 >= 5 {
                    lb = chosen_rect.y + chosen_rect.h / 2;
                    ub = chosen_rect.y + chosen_rect.h / 2 + 1;
                }else if chosen_rect.h / 2 < 5{
                    structure_num -= 1;
                    continue;
                }

                //println!("lower bound {} upper bound {}", lb, ub);

                split_y = rng.gen_range(lb, ub);
            }
            temp.append(&mut chosen_rect.split(split_x, split_y));
        }

        rect_squad.remove(randi);
        rect_squad.append(&mut temp);
    }
    
    //println!("rectangles split up!");

    for mut rect in rect_squad {
        //println!("shrinking rectangles");

        let mut rng = rand::thread_rng();
        let shrinkage = rng.gen_range(2 + (10 - structure_num), 5 + 1 * (10 - structure_num));

        //println!("shrinkage {}", shrinkage);

        rect.shrink((shrinkage as f32 * 1.5) as usize, shrinkage);

        area.structures.push(rect.clone());
        //println!("transferring rect tiles..");

        let mut perimeter = Vec::new();
        for y in rect.y..(rect.y + rect.h) {
            for x in rect.x..(rect.x + rect.w) {
                //println!("replacing tile at {:?}", (x, y));
                if (x == rect.x || x == rect.x + rect.w - 1) || (y == rect.y || y == rect.y + rect.h - 1) {
                    if !((x == rect.x || x == rect.x + rect.w - 1) && (y == rect.y || y == rect.y + rect.h - 1)) {
                        perimeter.push((x, y));
                    }
                    area.tiles[x + y * map_width] = TileBlock::new(Tile::WoodWall, false);
                }else{
                    area.tiles[x + y * map_width] = TileBlock::new(Tile::WoodFloor, true);
                }
                
            }
        }

        //println!("tiles transferred!");

        let randi = rng.gen_range(0, perimeter.len());
        
        //println!("door tile index selected! {}", randi);

        let (doorx, doory) = perimeter[randi];

        //println!("door tile is {:?}", (doorx, doory));

        area.tiles[doorx + doory * map_width] = TileBlock::new(Tile::WoodFloor, true);

        //println!("door placed!");
    }

    //println!("structures generated!");
}

pub fn load_map(map: &mut Map, to_load: (Option<Area>, usize)) {
    //println!("loading area to map from {}", to_load.1);

    let mut area_pointer = &mut (Area::new());
    match to_load.0 {
        Some(a) => {
            map.world_map.push(a);
            area_pointer = &mut map.world_map[to_load.1];
        }
        None => {
            area_pointer = &mut map.world_map[to_load.1];
        }
    }

    for i in 0..map.tiles.len() {
        map.tiles[i] = (*area_pointer).tiles[i];
    }
    for i in 0..map.anchor_points.len() {
        map.anchor_points[i] = (*area_pointer).anchor_points[i].clone();
    }

    map.structures = (*area_pointer).structures.clone();
    map.spawned = (*area_pointer).spawned;
    (*area_pointer).spawned = true;
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
    
    //println!("area lookup is {}", *new_area_index);


    if *new_area_index == usize::max_value() {
        //println!("creating new area");

        let mut area = Area::new();

        let mut rng = rand::thread_rng();

        let w = map.width;
        let h = map.height;
        

        let perlin = Perlin::new();
        let billow = Billow::new();

        //perlin.set_seed(rng.gen::<u32>());
        let bxseed = map.world_seed.4;
        let byseed = map.world_seed.5;
        let xseed = map.world_seed.0;
        let yseed = map.world_seed.1;
        let aseed = map.world_seed.2;
        let bseed = map.world_seed.3;


        for y in 0..h {
            for x in 0..w {
                let noise = perlin.get([xseed + x as f64 / w as f64 / ZOOM_FACTOR, yseed + y as f64 / h as f64 / ZOOM_FACTOR]);
                let adjustment = billow.get([aseed + x as f64 / w as f64 / ADJUSTMENT_ZOOM_FACTOR, bseed + y as f64 / h as f64 / ADJUSTMENT_ZOOM_FACTOR]) * NOISE_DISPLACEMENT;
                //(rng.gen::<f64>() - rng.gen::<f64>()) * NOISE_DISPLACEMENT;//
                let biome_mod = (perlin.get([bxseed + (map.location.0 as f64 + x as f64 / w as f64 as f64) * BIOME_RESOLUTION_FACTOR as f64, byseed + (map.location.1 as f64 + y as f64 / h as f64) * BIOME_RESOLUTION_FACTOR as f64]).abs() * BIOME_NUM as f64) as u32;
                
                let mut tilefloat = noise_ease(noise + adjustment);
                
                if tilefloat >= 1.0 {
                    tilefloat = 0.99;
                }else if tilefloat < 0.0 {
                    tilefloat = 0.0;
                }

                let tile = num::FromPrimitive::from_u32(biome_mod * BIOME_TILESET_SIZE + (tilefloat * (BIOME_TILESET_SIZE as f64)) as u32).unwrap();
                area.tiles.push(TileBlock::new(tile, true));
            }
        }

        generate_structures(&mut area, map.location, (map.world_seed.6, map.world_seed.7), (map.width, map.height));
        
        for y in 0..h {
            for x in 0..w {
                map.tiles[x + y * w] = area.tiles[x + y * w];
            }
        }


        //adding out of bounds anchor points
        let west = Anchor::new(usize::max_value(), 0, map.location.0, map.location.1);
        let east = Anchor::new(w, 0, map.location.0, map.location.1);
        let north = Anchor::new(0, h, map.location.0, map.location.1);
        let south = Anchor::new(0, usize::max_value(), map.location.0, map.location.1);

        area.anchor_points.push(west);
        area.anchor_points.push(east);
        area.anchor_points.push(north);
        area.anchor_points.push(south);

        for ty in 0..h {
            for tx in 0..w {
                let mut anchor = Anchor::new(tx, ty, map.location.0, map.location.1);
                if tx == 0 {
                    //add west to succ
                    anchor.succ.push((0, 10));
                }else if tx == w - 1 {
                    //add east to succ
                    anchor.succ.push((1, 10));
                }
                if ty == 0 {
                    //add south to succ
                    anchor.succ.push((3, 10));
                } else if ty == h - 1 {
                    //add north to succ
                    anchor.succ.push((2, 10));
                }
                if map.tiles[tx + ty * w].passable {
                    for y in -1..2 {
                        let py = anchor.pos.1 as i32 + y;
                        if py < 0 {
                            continue;
                        }
                        let ny = py as usize;
                        if ny >= h {
                            break;
                        }
                        for x in -1..2 {
                            if x == 0 && y == 0 {
                                continue;
                            }
                            let px = anchor.pos.0 as i32 + x;
                            if px < 0 {
                                continue;
                            }
                            let nx = px as usize;
                            if nx >= w {
                                break;
                            }
                            let index = nx + ny * w;
                            let mut cost = 10;
                            if x != 0 && y != 0 {
                                cost = 14;
                            }
                            if map.tiles[index].passable {
                                anchor.succ.push((index + 4, cost));
                            }
                        }
                    }
                }
                area.anchor_points.push(anchor);
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
        //println!("no new area needed; found area {}", *new_area_index);

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

    let bxseed = map.world_seed.4;
    let byseed = map.world_seed.5;
    let xseed = map.world_seed.0;
    let yseed = map.world_seed.1;
    let aseed = map.world_seed.2;
    let bseed = map.world_seed.3;

    println!("map dim: {:?}", (w, h));

    for y in 0..h {
        for x in 0..w {
            let noise = perlin.get([xseed + x as f64 / w as f64 / ZOOM_FACTOR, yseed + y as f64 / h as f64 / ZOOM_FACTOR]);
            let adjustment = billow.get([aseed + x as f64 / w as f64 / ADJUSTMENT_ZOOM_FACTOR, bseed + y as f64 / h as f64 / ADJUSTMENT_ZOOM_FACTOR]) * NOISE_DISPLACEMENT;//(rng.gen::<f64>() - rng.gen::<f64>()) * NOISE_DISPLACEMENT;

            let biome_mod = (perlin.get([bxseed + (map.location.0 as f64 + x as f64 / w as f64 as f64) * BIOME_RESOLUTION_FACTOR as f64, byseed + (map.location.1 as f64 + y as f64 / h as f64) * BIOME_RESOLUTION_FACTOR as f64]).abs() * BIOME_NUM as f64) as u32;

            let mut tilefloat = noise_ease(noise + adjustment);
            
            if tilefloat >= 1.0 {
                tilefloat = 0.99;
            }else if tilefloat < 0.0 {
                tilefloat = 0.0;
            }

            let tile = num::FromPrimitive::from_u32(biome_mod * BIOME_TILESET_SIZE + (tilefloat * (BIOME_TILESET_SIZE as f64)) as u32).unwrap();
            //println!("tile {:?}", tile);
            area.tiles.push(TileBlock::new(tile, true));
            
            
        }
    }

    generate_structures(&mut area, map.location, (map.world_seed.6, map.world_seed.7), (map.width, map.height));

    map.structures = area.structures.clone();
    map.spawned = area.spawned;
    area.spawned = true;

    for y in 0..h {
        for x in 0..w {
            map.tiles[x + y * w] = area.tiles[x + y * w];
        }
    }

    println!("map tiles transferred");

    //adding out of bounds anchor points
    let west = Anchor::new(usize::max_value(), 0, map.location.0, map.location.1);
    let east = Anchor::new(w, 0, map.location.0, map.location.1);
    let north = Anchor::new(0, h, map.location.0, map.location.1);
    let south = Anchor::new(0, usize::max_value(), map.location.0, map.location.1);

    map.anchor_points.push(west.clone());
    map.anchor_points.push(east.clone());
    map.anchor_points.push(north.clone());
    map.anchor_points.push(south.clone());
    area.anchor_points.push(west);
    area.anchor_points.push(east);
    area.anchor_points.push(north);
    area.anchor_points.push(south);

    for ty in 0..h {
        for tx in 0..w {
            let mut anchor = Anchor::new(tx, ty, map.location.0, map.location.1);
            if tx == 0 {
                //add west to succ
                anchor.succ.push((0, 10));
            }else if tx == w - 1 {
                //add east to succ
                anchor.succ.push((1, 10));
            }
            if ty == 0 {
                //add south to succ
                anchor.succ.push((3, 10));
            } else if ty == h - 1 {
                //add north to succ
                anchor.succ.push((2, 10));
            }
            for y in -1..2 {
                let py = anchor.pos.1 as i32 + y;
                if py < 0 {
                    continue;
                }
                let ny = py as usize;
                if ny >= h {
                    break;
                }
                for x in -1..2 {
                    if x == 0 && y == 0 {
                        continue;
                    }
                    let px = anchor.pos.0 as i32 + x;
                    if px < 0 {
                        continue;
                    }
                    let nx = px as usize;
                    if nx >= w {
                        break;
                    }
                    let index = nx + ny * w;
                    let mut cost = 10;
                    if x != 0 && y != 0 {
                        cost = 14;
                    }
                    //println!("made it here to index {}", index);
                    if map.tiles[index].passable {
                        anchor.succ.push((index + 4, cost));
                    }
                }
            }
            map.anchor_points.push(anchor.clone());
            area.anchor_points.push(anchor);           
        }
    }

    map.world_map.push(area);
}

fn initialise_spritesheet_handles(world: &mut World) {
    println!("initialising spritesheet handles");
    
    let mut handles = SpriteSheetHandles::new();

    handles.add(SpriteSheetLabel::Particles, load_sprite_sheet(world, "particles"));
    handles.add(SpriteSheetLabel::Person, load_sprite_sheet(world, "player"));
    handles.add(SpriteSheetLabel::Tiles, load_sprite_sheet(world, "tiles"));
    handles.add(SpriteSheetLabel::UiTiles, load_sprite_sheet(world, "ui_tile"));
    handles.add(SpriteSheetLabel::Plants, load_sprite_sheet(world, "plants"));

    world.insert(handles);

    println!("spritesheet handles initialised");
}

fn initialise_ui_system(world: &mut World) {
    let ui_holder = UiHolder::new();

    world.insert(ui_holder);

    let ui_state = UiState::new();

    world.insert(ui_state);
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

            let tile = tiles[x + y * tile_num_w].tile;
            
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
        .with(components::Id::new(EntityType::Person))
        .with(components::Physical::new((s_w / 2.0, s_h / 2.0), (0, 0)))
        .with(local_transform)
        .build();
}

pub fn spawn_person(cux: usize, cuy: usize, ax: i32, ay: i32, handles: &Read<SpriteSheetHandles>, ents: &mut Entities, phys: &mut WriteStorage<components::Physical>, 
    movers: &mut WriteStorage<components::Mover>, ids: &mut WriteStorage<Id>, offs: &mut WriteStorage<components::Offscreen>, 
    trans: &mut WriteStorage<Transform>, srs: &mut WriteStorage<SpriteRender>, hungs: &mut WriteStorage<components::Hunger>) {
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(-100.0, 0.0, 0.0);

    let local_physical = components::Physical::new(((cux * TILE_SIZE + TILE_SIZE / 2) as f32, (cuy * TILE_SIZE + TILE_SIZE / 2) as f32), (ax, ay));
    let local_render = SpriteRender {
        sprite_sheet: handles.get(SpriteSheetLabel::Person).unwrap().clone(),
        sprite_number: 1,
    };
    let local_ids = Id::new(EntityType::Person);
    let local_mover = components::Mover::new(DEFAULT_BASE_SPEED);
    let local_off = components::Offscreen::new();

    let mut rng = rand::thread_rng();
    let hung = rng.gen::<f32>() * 0.5 + 0.5;

    let local_hunger = components::Hunger::new(DEFAULT_HUNGER_CAPACITY, DEFAULT_HUNGER_RATE, hung * DEFAULT_HUNGER_CAPACITY);

    ents.build_entity()
        .with(local_transform, trans)
        .with(local_physical, phys)
        .with(local_render, srs)
        .with(local_ids, ids)
        .with(local_mover, movers)
        .with(local_off, offs)
        .with(local_hunger, hungs)
        .build();
}

pub fn spawn_plant(cux: usize, cuy: usize, ax: i32, ay: i32, handles: &Read<SpriteSheetHandles>, ents: &mut Entities, phys: &mut WriteStorage<components::Physical>, plants: &mut WriteStorage<components::Plant>, ids: &mut WriteStorage<Id>, offs: &mut WriteStorage<components::Offscreen>, trans: &mut WriteStorage<Transform>, srs: &mut WriteStorage<SpriteRender>) {
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(-100.0, 0.0, 0.0);

    let local_physical = components::Physical::new(((cux * TILE_SIZE + TILE_SIZE / 2) as f32, (cuy * TILE_SIZE + TILE_SIZE / 2) as f32), (ax, ay));
    
    println!("handles Plants -> {:?}", handles.get(SpriteSheetLabel::Plants));

    
    let local_render = SpriteRender {
        sprite_sheet: handles.get(SpriteSheetLabel::Plants).unwrap().clone(),
        sprite_number: 0,
    };
    let local_ids = Id::new(EntityType::Plant);
    let local_off = components::Offscreen::new();

    let mut rng = rand::thread_rng();
    let ripeness = rng.gen::<f32>();
    let fruit_rate = rng.gen::<f32>() * 0.4 + 0.8;

    let local_plant = components::Plant::new(true, fruit_rate, ripeness);

    ents.build_entity()
        .with(local_transform, trans)
        .with(local_physical, phys)
        .with(local_render, srs)
        .with(local_plant, plants)
        .with(local_ids, ids)
        .with(local_off, offs)
        .build();
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
            map.world_seed = (rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64, rng.gen::<f64>() * MAP_SEED_RANGE as f64);

            println!("Loaded config: {:?}", loaded);
            data.world.insert(loaded);
            data.world.insert(map);


            let world = &mut data.world;

            initialise_spritesheet_handles(world);

            generate_map(*world);

            self.sprite_sheet_handle.replace(load_sprite_sheet(*world, "tiles"));
    
            initialise_tiles(*world, self.sprite_sheet_handle.clone().unwrap());
            
            self.sprite_sheet_handle.replace(load_sprite_sheet(*world, "player"));
            
            initialise_player(*world, self.sprite_sheet_handle.clone().unwrap());
    
            //initialise_persons(world, self.sprite_sheet_handle.clone().unwrap());
            
            initialise_ui_system(world);

            

            initialise_camera(*world);

            //storage of screen dimensions
            data.world.insert(Dimensions::new());


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

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let mut dim = data.world.fetch_mut::<Dimensions>();
        dim.width = data.world.fetch::<ScreenDimensions>().width();
        dim.height = data.world.fetch::<ScreenDimensions>().height();
        Trans::None
    }
}