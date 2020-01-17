use amethyst::ecs::prelude::{Component, VecStorage};
use crate::game_state::TILE_SIZE;

#[derive(Clone)]
pub struct Physical{
    real_pos: (f32, f32),
    area_pos: (i32, i32),
    offset: (f32, f32),
}

impl Physical{
    pub fn new(real_pos: (f32, f32), area_pos: (i32, i32)) -> Physical {
        Physical {
            real_pos,
            area_pos,
            offset: (0.0, 0.0),
        }
    }
    pub fn get_location(&self) -> (i32, i32) {
        self.area_pos
    }
    pub fn mut_area_x(&mut self, i: i32) {
        self.area_pos.0 += i;
    }
    pub fn mut_area_y(&mut self, i: i32) {
        self.area_pos.1 += i;
    }
    pub fn set_area_x(&mut self, x: i32) {
        self.area_pos.0 = x;
    }
    pub fn set_area_y(&mut self, y: i32) {
        self.area_pos.1 = y;
    }
    pub fn get_real_position(&self) -> (f32, f32) {
        self.real_pos
    }
    pub fn get_display_offset(&self) -> (f32, f32) {
        self.offset
    }
    pub fn set_x(&mut self, new_x: f32) {
        self.real_pos.0 = new_x;
    }
    pub fn set_y(&mut self, new_y: f32) {
        self.real_pos.1 = new_y;
    }
    pub fn mut_x(&mut self, d_x: f32) {
        self.real_pos.0 += d_x;
    }
    pub fn mut_y(&mut self, d_y: f32) {
        self.real_pos.1 += d_y;
    }
    /*pub fn set_tile_position(&mut self, real_position: (f32, f32)) -> (usize, usize) {
        let mut tile_x = 0;
        let mut tile_y = 0;

        tile_x = (real_position.0 as usize) / TILE_SIZE;
        tile_y = (real_position.1 as usize) / TILE_SIZE;

        self.position = (tile_x, tile_y);
        self.position
    }*/
    pub fn into_tile_position(real_position: (f32, f32)) -> (usize, usize) {
        let mut tile_x = 0;
        let mut tile_y = 0;

        tile_x = (real_position.0 as usize) / TILE_SIZE;
        tile_y = (real_position.1 as usize) / TILE_SIZE;

        (tile_x, tile_y)
    }
    pub fn get_tile_position(&self) -> (usize, usize) {
        Physical::into_tile_position(self.real_pos)
    }
         
}

impl Component for Physical{
    type Storage = VecStorage<Self>;
}