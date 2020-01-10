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
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};


use crate::components::{Id, Particle, ParticleDeathType};

pub const PLAYER_WIDTH: usize = 1;
pub const PLAYER_HEIGHT: usize = 1;
pub const PLAYER_SPEED: f32 = 120.0;

pub const PERSON_NUM: u32 = 25;

pub const DEFAULT_BASE_SPEED: f32 = 120.0;

pub const TILE_SIZE: usize = 16;
pub const ENTITY_LIM: usize = 50;

pub const ZOOM_FACTOR: f64 = 0.5;
pub const ADJUSTMENT_ZOOM_FACTOR: f64 = 0.25;
pub const NOISE_DISPLACEMENT: f64 = 0.5;

pub const OFFSCREEN_UNKNOWN_PATH_WAIT_TIME: f32 = 10.0 * 100.0 / DEFAULT_BASE_SPEED; //in seconds

pub const RUNE_BOARD_DIM: usize = 3;
pub const RUNE_BOARD_TILE_SIZE: usize = 64;

pub const MAX_BRUSH_STROKE_DIST: f32 = 4.0;
pub const STROKE_FORGIVENESS: f32 = RUNE_BOARD_TILE_SIZE as f32 / 4.0;

pub const RUNE_ALPHABET_SIZE: usize = 1;
pub const MAX_RUNE_EDGES: usize = 5;

#[derive(Debug, Clone, Copy)]
pub enum SpellComponent {
    Force,
    Fire,
    Size,
}

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub rune_board_rune: Option<Rune>,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            rune_board_rune: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuneAlphabet {
    letters: Vec<Rune>,
}

impl Default for RuneAlphabet {
    fn default() -> Self {
        RuneAlphabet::new()
    }
}

impl RuneAlphabet {
    pub fn new() -> Self {
        RuneAlphabet {
            letters: Vec::new(),
        }
    }
    pub fn add_rune(&mut self, rune: Rune) -> bool {
        let mut out = false;
        if !self.letters.contains(&rune) {
            self.letters.push(rune);
            out = true;
        }
        out
    }    
    pub fn is_in(&self, rune: Rune) -> bool {
        self.letters.contains(&rune)
    }
    pub fn len(&self) -> usize {
        self.letters.len()
    }
}

#[derive(Debug, Clone)]
pub struct Rune {
    pub edges: Vec<((usize, usize),(usize, usize))>,
    pub flavour: SpellComponent,
}

impl Default for Rune {
    fn default() -> Self {
        Rune::new()
    }
}

impl Rune {
    pub fn new() -> Self {
        Rune {
            edges: Vec::new(),
            flavour: SpellComponent::Size,
        }
    }
    pub fn add_edge(&mut self, pt: ((usize, usize),(usize, usize))) -> bool {
        let mut out = false;
        let pt_a = pt.0;
        let pt_b = pt.1;
        
        let mut invalid = false;
        
        if (pt_a.0 as i32 - pt_b.0 as i32).abs() > 1 || (pt_a.1 as i32 - pt_b.1 as i32).abs() > 1 {
            invalid = true;
        }else{
        
            if pt_a != pt_b {
                for (ea, eb) in self.edges.iter() {
                    let mut found_a = false;
                    let mut found_b = false;    
                    if pt_a == *ea || pt_a == *eb {
                        found_a = true;
                    }
                    if pt_b == *ea || pt_b == *eb {
                        found_b = true;
                    }
                    if found_a && found_b {
                        invalid = true;
                    }
                }
            }else{
                invalid = self.edges.contains(&pt);
            }
        }
        if !invalid {
            self.edges.push(pt);
            out = true;
        }
        out
    }
}

impl PartialEq for Rune {
    fn eq(&self, other: &Rune) -> bool {
        if self.edges.len() != other.edges.len() {
            false  
        }else{
            let mut out = true;
            for pt in self.edges.iter() {
                if !other.edges.contains(pt) {
                    out = false;
                    break;
                }
            }
            if out {
                for pt in other.edges.iter() {
                    if !self.edges.contains(pt) {
                        out = false;
                        break;
                    }
                }
            }
            out
        }
    }
}

#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum Tile {
    GrassyHeavy = 0,
    Grassy,
    Plain,
    RockyLight,
    Rocky,
    Size
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
    pub tiles: Vec<TileBlock>,
    pub anchor_points: Vec<Anchor>,
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
            tiles: vec![TileBlock::new(Tile::Size, true); width * height],
            anchor_points: Vec::new(),
            entities: vec![Id::nil(); ENTITY_LIM],
            world_map: Vec::new(),
            area_index: 0,
            rerolled: false,
        }   
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum SpriteSheetLabel {
    Particles,
    Person,
    Tiles,
    UiTiles,
    Size,
}

impl Default for SpriteSheetLabel {
    fn default() -> Self {
        SpriteSheetLabel::Size
    }
}

#[derive(Default)]
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
        self.handles.push((label, handle));
    }
    pub fn get(&self, label: SpriteSheetLabel) -> Option<Handle<SpriteSheet>> {
        let mut out = None;
        for (tag, handle) in self.handles.iter() {
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

pub fn load_map(map: &mut Map, to_load: (Option<Area>, usize)) {
    //println!("loading area to map from {}", to_load.1);

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
    for i in 0..map.anchor_points.len() {
        map.anchor_points[i] = (*area_pointer).anchor_points[i].clone();
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
                area.tiles.push(TileBlock::new(tile, true));
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

pub fn display_rune(rune: Rune, x: f32, y: f32, ui_index: usize, handles: &Read<SpriteSheetHandles>, ents: &mut Entities, parts: &mut WriteStorage<Particle>, trans: &mut WriteStorage<Transform>, srs: &mut WriteStorage<SpriteRender>) {
    for ((ox, oy), (ex, ey)) in rune.edges.iter() {
        let dx = *ex as i32 - *ox as i32;
        let dy = *ey as i32 - *oy as i32;

        let mut px = x - 32.0;//config.stage_width / 3.0 - 32.0;
        let mut py = y - 32.0;;//config.stage_height * 2.0 / 3.0 - 32.0;

        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(0.0, 0.0, 3.0);


        let mut sprite_render = SpriteRender {
            sprite_sheet: handles.get(SpriteSheetLabel::Particles).unwrap(),
            sprite_number: 1,
        };

        if dx == 0 && dy == 0 {
            //point
            //no rotation/transform needed just translation
            px += (*ox) as f32 * 32.0;
            py += (*oy) as f32 * 32.0;
            
        }else if dx != 0 && dy == 0 {

            local_transform.set_rotation_z_axis((PI / 2.0) as f32);
            px += (*ox) as f32 * 32.0 + 16.0;
            py += (*oy) as f32 * 32.0;
            if dx < 0 {
                px -= 32.0;
                //local_transform.set_rotation_z_axis(PI as f32);
            }
            sprite_render.sprite_number = 2;

        }else if dx == 0 && dy != 0 {

            px += (*ox) as f32 * 32.0;
            py += (*oy) as f32 * 32.0 - 16.0;
            if dy > 0 {
                py += 32.0;
            }
            sprite_render.sprite_number = 2;

        }else {
            //full diagonal
            if dx * dy > 0 {
                local_transform.set_rotation_z_axis((PI / 2.0) as f32);
            }
            px += (*ox) as f32 * 32.0 + 16.0;
            if dx < 0 {
                px -= 32.0;
            }
            py += (*oy) as f32 * 32.0 - 16.0;
            if dy > 0 { 
                py += 32.0;
            }
            sprite_render.sprite_number = 3; 

        }

        let mut local_particle = Particle::new(px, py, 0, ParticleDeathType::Ui);
        local_particle.set_ui(ui_index);

        ents.build_entity()
            .with(local_transform, trans)
            .with(local_particle, parts)
            .with(sprite_render, srs)
            .build();
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

    println!("map dim: {:?}", (w, h));

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
            area.tiles.push(TileBlock::new(tile, true));
            map.tiles[x + y * w] = area.tiles[x + y * w];
        }
    }

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
    /*
    for i in 0..(map.width * map.height) {
        area.tiles.push(num::FromPrimitive::from_u32(rng.gen_range(0, Tile::Size as u32)).unwrap());
        map.tiles[i] = area.tiles[i];
    }*/

    map.world_map.push(area);
}

fn initialise_runes(world: &mut World) {
    let mut rune_alphabet = RuneAlphabet::new();
    while rune_alphabet.len() < RUNE_ALPHABET_SIZE {
        let mut rune = Rune::new();
        let mut edge_calls = 0;

        while edge_calls < MAX_RUNE_EDGES {
            let mut rng = rand::thread_rng();

            let ox = rng.gen_range(0, RUNE_BOARD_DIM);
            let oy = rng.gen_range(0, RUNE_BOARD_DIM);

            let dx = rng.gen_range(-1, 2);
            let dy = rng.gen_range(-1, 2);

            let ex = (ox as i32 + dx).max(0).min(RUNE_BOARD_DIM as i32 - 1) as usize;
            let ey = (oy as i32 + dy).max(0).min(RUNE_BOARD_DIM as i32 - 1) as usize;

            rune.add_edge( ((ox, oy), (ex, ey)) );
            edge_calls += 1;
        }

        rune_alphabet.add_rune(rune);
    }

    println!("generated Rune Alphabet is {:?}", rune_alphabet);

    world.insert(rune_alphabet);
}

fn initialise_spritesheet_handles(world: &mut World) {
    let mut handles = SpriteSheetHandles::new();

    handles.add(SpriteSheetLabel::Particles, load_sprite_sheet(world, "particles"));
    handles.add(SpriteSheetLabel::Person, load_sprite_sheet(world, "player"));
    handles.add(SpriteSheetLabel::Tiles, load_sprite_sheet(world, "tiles"));
    handles.add(SpriteSheetLabel::UiTiles, load_sprite_sheet(world, "ui_tile"));

    world.insert(handles);
}

fn initialise_ui_system(world: &mut World) {
    let ui_holder = UiHolder::new();

    world.insert(ui_holder);

    let ui_state = UiState::new();

    world.insert(ui_state);
}

fn initialise_rune_board_ui(world: &mut World, sprite_sheet: Handle<SpriteSheet>){
    let s_w = world.read_resource::<Config>().stage_width;
    let s_h = world.read_resource::<Config>().stage_height;
    
    let offset_x = s_w * 2.0 / 3.0 - 96.0;
    let offset_y = s_h * 2.0 / 3.0 - 96.0;

    let ui_len = world.read_resource::<UiHolder>().len();

    world.write_resource::<UiHolder>().add_ui(Ui::RuneBoard);
    

    for y in 0..RUNE_BOARD_DIM {
        for x in 0..RUNE_BOARD_DIM {
            let mut local_transform = Transform::default();
            local_transform.set_translation_xyz(offset_x + (x * RUNE_BOARD_TILE_SIZE) as f32, offset_y + (y * RUNE_BOARD_TILE_SIZE) as f32, 1.0);
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0,
            };

            world
                .create_entity()
                .with(sprite_render)
                .with(local_transform)
                .with(components::SubUi::new(offset_x + (x * RUNE_BOARD_TILE_SIZE) as f32, offset_y + (y * RUNE_BOARD_TILE_SIZE) as f32, 64.0, 64.0, ui_len))
                .build();
        }
    }

    
    let offset_x = s_w * 1.0 / 3.0 - 64.0;
    let offset_y = s_h * 2.0 / 3.0 - 64.0;

    let ui_len = world.read_resource::<UiHolder>().len();

    world.write_resource::<UiHolder>().add_ui(Ui::RuneDisplay);

    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(offset_x, offset_y, 1.0);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 1,
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(local_transform)
        .with(components::SubUi::new(offset_x, offset_y, 128.0, 128.0, ui_len))
        .build();
    
    world.write_resource::<UiHolder>().add_ui(Ui::SpellDisplay);
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
            .with(components::Mover::new(DEFAULT_BASE_SPEED))
            .with(components::Offscreen::new())
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
            
            initialise_ui_system(world);

            self.sprite_sheet_handle.replace(load_sprite_sheet(*world, "ui_tile"));

            initialise_rune_board_ui(world, self.sprite_sheet_handle.clone().unwrap());

            initialise_spritesheet_handles(world);

            initialise_runes(world);

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