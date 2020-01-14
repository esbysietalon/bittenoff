use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    winit::MouseButton,
    renderer::SpriteRender,
    window::ScreenDimensions,
};
use amethyst::ecs::prelude::{Entity, Entities};
use crate::game_state::{UiHolder, UiState, Ui, SpriteSheetHandles, SpriteSheetLabel, 
    Config, Dimensions, KeyCheck};
use crate::components::{SubUi, Particle, ParticleDeathType};

use std::f32::consts::PI;

pub struct UiDisplaySystem;

impl<'s> System<'s> for UiDisplaySystem{
    type SystemData = (
        Read<'s, UiHolder>,
        Write<'s, UiState>,
        WriteStorage<'s, SubUi>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Particle>,
        Entities<'s>,
        Read<'s, SpriteSheetHandles>,
        Read<'s, Config>,
    );

    fn run(&mut self, (ui_holder, mut ui_state, mut eles, mut trans, mut srs, mut parts, mut ents, handles, config): Self::SystemData) {
        
        for (ele, tran) in (&mut eles, &mut trans).join() {
            if ui_holder.is_active(ele.parent()) {
                tran.set_translation_xyz(ele.get_x() + ele.get_w() / 2.0, ele.get_y() + ele.get_h() / 2.0, 1.0);
            }else{
                tran.set_translation_xyz(0.0, 0.0, 100.0);
            }
        }

        //rune board
        //println!("ui_state: {:?}", *ui_state);
        for ui_index in 0..ui_holder.len() {
            if ui_holder.is_active(ui_index) {
               
                
            }
        }

    }
}

pub struct UiControlSystem{
    pub input_ready: bool,
    pub time_counter: f32,
}

impl UiControlSystem {
    pub fn new() -> UiControlSystem {
        UiControlSystem {
            input_ready: true,
            time_counter: 0.0,
        }
    }
}

impl<'s> System<'s> for UiControlSystem{
    type SystemData = (
        Read<'s, UiHolder>,
        Write<'s, UiState>,
        WriteStorage<'s, SubUi>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Particle>,
        Entities<'s>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>,
        Read<'s, Config>,
        Read<'s, SpriteSheetHandles>,
        Read<'s, Dimensions>,
    );

    fn run(&mut self, (ui_holder, mut ui_state, mut eles, mut trans, mut srs, mut parts, mut entities, input, time, config, sprites, dim): Self::SystemData) {
        self.time_counter += time.delta_seconds();
        

        if !input.mouse_button_is_down(MouseButton::Left) {
            self.input_ready = true;            
        }


        for ele in (&mut eles).join() {
            if ui_holder.is_active(ele.parent()) {
                //println!("ele active");
                match ui_holder.get_type(ele.parent()) {
                    _ => {}
                }
            }
            
        }
        
    }
}
