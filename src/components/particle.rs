use amethyst::ecs::prelude::{Component, VecStorage};

#[derive(Clone, Copy)]
pub enum DeathType {
    Ui = 0,
    Size,
}

pub struct Particle{
    sprite: usize,
    x: f32,
    y: f32,
    death_type: DeathType, 
    lifespan: f32,
    ui: usize,
    key_check_index: Option<usize>,
}

impl Particle {
    pub fn new(x: f32, y: f32, sprite: usize, death_type: DeathType) -> Particle{
        Particle{
            sprite,
            x,
            y,
            death_type,
            lifespan: 0.0,
            ui: 0,
            key_check_index: None,
        }
    }
    pub fn get_x(&self) -> f32 {
        self.x
    }
    pub fn get_y(&self) -> f32 {
        self.y
    }
    pub fn mut_x(&mut self, i: f32) {
        self.x += i;
    }
    pub fn mut_y(&mut self, i: f32) {
        self.y += i;
    }
    pub fn set_ui(&mut self, i: usize) {
        self.ui = i;
    }
    pub fn set_lifespan(&mut self, i: f32) {
        self.lifespan = i;
    }
    pub fn mut_lifespan(&mut self, i: f32) {
        self.lifespan += i;
    }
    pub fn get_ui(&self) -> usize {
        self.ui
    }
    pub fn get_key_check(&self) -> Option<usize> {
        self.key_check_index
    }
    pub fn set_key_check(&mut self, i: usize) {
        self.key_check_index = Some(i);
    }
    pub fn get_sprite(&self) -> usize {
        self.sprite
    }
    pub fn get_death_type(&self) -> DeathType {
        self.death_type
    }
    pub fn get_lifespan(&self) -> f32 {
        self.lifespan
    }
}

impl Component for Particle {
    type Storage = VecStorage<Self>;
}