use amethyst::{
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Join, Read, Write, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};
use crate::components::{Player, Physical};
use crate::game_state::{Config, UiHolder, UiState, Map, Area, load_map, regenerate_map, update_world_seed, PLAYER_SPEED};

pub struct MapSystem;

impl<'s> System<'s> for MapSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Physical>,
        Read<'s, Config>,
        Write<'s, Map>,
    );

    fn run(&mut self, (players, mut physicals, config, mut map): Self::SystemData) {
        let mut change_map = false;
        //let mut area_pointer = &mut Area::new();
        let mut area_index = map.area_index;
        
        /*{
            let curr_area_index = map.area_index;
            area_pointer = &mut (map.world_map[curr_area_index])
        }*/

        let mut dir = ' ';

        for (player, phys) in (&players, &mut physicals).join(){
            let (x, y) = phys.get_real_position();
            

            if x > config.stage_width as f32 {
                phys.set_x(0.0);
                phys.mut_area_x(1);
                dir = 'e';
                change_map = true;
            }else if x < 0.0 {
                phys.set_x(config.stage_width as f32);
                phys.mut_area_x(-1);
                dir = 'w';
                change_map = true;
            }else if y > config.stage_height as f32 {
                phys.set_y(0.0);
                phys.mut_area_y(1);
                dir = 'n';
                change_map = true;
            }else if y < 0.0 {
                phys.set_y(config.stage_height as f32);
                phys.mut_area_y(-1);
                dir = 's';
                change_map = true;
            }

            if change_map {
                break;
            }    
        }
        if change_map {
            //println!("change map! {}", dir);

            {
                update_world_seed(&mut map, dir);
            }

            let mut load_tuple = (None, 0);

            {
                load_tuple = regenerate_map(&mut map, area_index, dir);
            }

            load_map(&mut map, load_tuple);
        }
    }
}

pub struct ActionSystem{
    pub input_ready: bool,
}

impl ActionSystem {
    pub fn new() -> ActionSystem {
        ActionSystem {
            input_ready: true,
        }
    }
}

impl<'s> System<'s> for ActionSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
        Write<'s, UiHolder>,
        Write<'s, UiState>,
        Read<'s, Time>,
    );

    fn run(&mut self, (players, config, input, mut ui_holder, mut ui_state, time): Self::SystemData) {
        for (player) in (&players).join() {
            let action = input.action_is_down("action").unwrap_or(false);

            if action {
                if self.input_ready {
                    let curr = ui_holder.is_active(0);
                    
                    ui_state.rune_board_rune = None;
                    //Rune Craft UI
                    ui_holder.set_active(0, !curr);
                    ui_holder.set_active(1, !curr);
                    ui_holder.set_active(2, !curr);
                }
                self.input_ready = false;
            }else {
                self.input_ready = true;
            }
            
        }
    }
}
pub struct MoveSystem;

impl<'s> System<'s> for MoveSystem{
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Physical>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>
    );

    fn run(&mut self, (players, mut physicals, config, input, time): Self::SystemData) {
        

        for (player, phys) in (&players, &mut physicals).join(){
            let movement = input.axis_value("horizontal_mv");
            //println!("running, since {}", time.delta_seconds());
            if let Some(mv_amount) = movement {
                let scaled_amount = PLAYER_SPEED * time.delta_seconds() * mv_amount as f32;
                let x = phys.get_real_position().0;
                //println!("updating x by {}", scaled_amount);
                phys.set_x(
                    (x + scaled_amount)
                        //.min(config.stage_width as f32 - player.width as f32 * 0.5)
                        //.max(player.width as f32 * 0.5),
                );
            }


            let movement = input.axis_value("vertical_mv");
            if let Some(mv_amount) = movement {
                let scaled_amount = PLAYER_SPEED * time.delta_seconds() * mv_amount as f32;
                let y = phys.get_real_position().1;
                //println!("updating x by {}", scaled_amount);
                phys.set_y(
                    (y + scaled_amount)
                        //.min(config.stage_height as f32 - player.height as f32 * 0.5)
                        //.max(player.height as f32 * 0.5),
                );
            }
        }
    }
}