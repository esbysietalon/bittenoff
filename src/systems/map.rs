use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    renderer::{SpriteRender},
};
use crate::game_state::{Map};
use crate::components::{Tile};

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