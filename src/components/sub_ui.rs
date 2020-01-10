use amethyst::ecs::prelude::{Component, VecStorage};

#[derive(Clone, Copy, Debug)]
pub struct SubUi{
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    parent: usize,
}

impl SubUi {
    pub fn new(x: f32, y: f32, w: f32, h: f32, parent: usize) -> SubUi{
        SubUi{
            x,
            y,
            w,
            h,
            parent,
        }
    }
    pub fn set_x(&mut self, new_x: f32) {
        self.x = new_x;
    }
    pub fn set_y(&mut self, new_y: f32) {
        self.y = new_y;
    }
    pub fn get_center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
    pub fn get_x(&self) -> f32 {
        self.x
    }
    pub fn get_y(&self) -> f32 {
        self.y
    }
    pub fn get_w(&self) -> f32 {
        self.w
    }
    pub fn get_h(&self) -> f32 {
        self.h
    }
    pub fn parent(&self) -> usize {
        self.parent
    }
}

impl Component for SubUi {
    type Storage = VecStorage<Self>;
}

impl PartialEq for SubUi {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.parent == other.parent
    }
}