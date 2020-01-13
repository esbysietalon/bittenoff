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
    Config, Dimensions, Rune, RuneAlphabet, KeyCheck,
    display_rune,
    MAX_BRUSH_STROKE_DIST, STROKE_FORGIVENESS, RUNE_BOARD_DIM};
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
        Read<'s, RuneAlphabet>,
    );

    fn run(&mut self, (ui_holder, mut ui_state, mut eles, mut trans, mut srs, mut parts, mut ents, handles, config, runalp): Self::SystemData) {
        
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
                if ui_holder.get_type(ui_index) == Ui::SpellDisplay {
                    let sx = config.stage_width / 2.0 - 40.0 * ui_state.current_spell.len() as f32;
                    let mut i = 0;
                    for rune in ui_state.current_spell.iter() {
                        display_rune(
                            rune.clone(),
                            sx + 80.0 * i as f32, 
                            config.stage_height * 1.0 / 3.0,
                            0.75,
                            0.01,
                            None,
                            ui_index,
                            &handles,
                            &mut ents,
                            &mut parts,
                            &mut trans,
                            &mut srs
                        );
                        i += 1;
                    }
                }else if ui_holder.get_type(ui_index) == Ui::RuneDisplay {
                    match &ui_state.rune_board_rune {
                        None => {

                        }
                        Some(rune) => {
                            display_rune(
                                rune.clone(),
                                config.stage_width * 2.0 / 3.0, 
                                config.stage_height * 2.0 / 3.0,
                                2.0,
                                0.01,
                                Some(KeyCheck::Enter as usize),
                                ui_index,
                                &handles,
                                &mut ents,
                                &mut parts,
                                &mut trans,
                                &mut srs
                            );

                            if runalp.is_in(rune.clone()) {
                                display_rune(
                                    rune.clone(),
                                    config.stage_width * 1.0 / 3.0, 
                                    config.stage_height * 2.0 / 3.0,
                                    1.0,
                                    0.01,
                                    Some(KeyCheck::Enter as usize),
                                    ui_index,
                                    &handles,
                                    &mut ents,
                                    &mut parts,
                                    &mut trans,
                                    &mut srs
                                );
                            }
                        }
                    }
                }
            }
        }

    }
}

pub struct UiControlSystem{
    pub input_ready: bool,
    pub time_counter: f32,
    pub draw_points: Vec<(f32, f32)>,
    pub grid_points: Vec<((f32, f32), (usize, usize))>,
    pub stroked_tiles: Vec<(usize, usize)>,
    pub per_stroke_particles: Vec<Entity>,
}

impl UiControlSystem {
    pub fn new() -> UiControlSystem {
        UiControlSystem {
            input_ready: true,
            time_counter: 0.0,
            draw_points: Vec::new(),
            grid_points: Vec::new(),
            stroked_tiles: Vec::new(),
            per_stroke_particles: Vec::new(),
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
        
        let mut checked = false;

        if !input.mouse_button_is_down(MouseButton::Left) {
            self.input_ready = true;
            self.draw_points.clear();

            //clear stroke particles from last stroke
            for ent in self.per_stroke_particles.iter() {
                entities.delete(*ent);
            }

            let mut rune = Rune::new();
            match &ui_state.rune_board_rune {
                None => {
                    //
                }
                Some(r) => {
                    rune = r.clone();
                }
            }
            if self.stroked_tiles.len() > 1 {
                for i in 0..(self.stroked_tiles.len() - 1) {
                    rune.add_edge((self.stroked_tiles[i], self.stroked_tiles[i+1]));
                }
            }else if self.stroked_tiles.len() > 0 {
                rune.add_edge((self.stroked_tiles[0], self.stroked_tiles[0]));
            }            
            ui_state.rune_board_rune = Some(rune);

            self.stroked_tiles.clear();
        }

        self.grid_points.clear();

        let mut rb_i = 0;

        for ele in (&mut eles).join() {
            if ui_holder.is_active(ele.parent()) {
                //println!("ele active");
                match ui_holder.get_type(ele.parent()) {
                    Ui::RuneBoard => {
                        //println!("ele is runeboard");
                        if self.time_counter > 0.1 && input.mouse_button_is_down(MouseButton::Left) {
                            checked = true;
                            //println!("time_counter passed");
                            if self.input_ready {
                                self.draw_points.clear();
                                self.input_ready = false;
                            }

                            match input.mouse_position() {
                                None => {}
                                Some((mx, my)) => {
                                    let nmx = mx / dim.width * config.stage_width;
                                    let nmy = config.stage_height - my / dim.height * config.stage_height;

                                    let lb = ele.get_x();
                                    let rb = ele.get_x() + ele.get_w();
                                    let db = ele.get_y();
                                    let ub = ele.get_y() + ele.get_h();
                                    //println!("mouse position is {}, {} || target bounds lb {} rb {} db {} ub {}", nmx, nmy, lb, rb, db, ub);
                                    
                                    let iy = rb_i / RUNE_BOARD_DIM;
                                    let ix = rb_i - iy * RUNE_BOARD_DIM;

                                    self.grid_points.push((ele.get_center(), (ix, iy)));

                                    if nmx >= lb && nmx <= rb && nmy >= db && nmy <= ub {
                                        //println!("mouse in ele bounds");
                                        self.draw_points.push((nmx, nmy));
                                    }
                                }
                            }
                            
                        }

                        rb_i += 1;
                    }
                    _ => {}
                }
            }
            
        }
        if self.draw_points.len() > 0 {
            let mut ui_index = 0;
            for i in 0..ui_holder.len() {
                if ui_holder.get_type(i) == Ui::RuneBoard {
                    ui_index = i;
                }
            }
            let (mut lpx, mut lpy) = self.draw_points[0];
            for (px, py) in self.draw_points.iter() {
                let (mut cx, mut cy) = (lpx, lpy);
                
                //initial point

                for ((grx, gry), (gtx, gty)) in self.grid_points.iter() {
                    if (*grx - cx).abs() < STROKE_FORGIVENESS && (*gry - cy).abs() < STROKE_FORGIVENESS {
                        if !self.stroked_tiles.contains(&(*gtx, *gty)) {
                            self.stroked_tiles.push((*gtx, *gty));
                        }
                        break;
                    }
                }

                let mut local_transform = Transform::default();
                local_transform.set_translation_xyz(0.0, 0.0, 3.0);
                
                let mut local_particle = Particle::new(cx, cy, 0, ParticleDeathType::Ui);
                local_particle.set_ui(ui_index);

                let sprite_render = SpriteRender {
                    sprite_sheet: sprites.get(SpriteSheetLabel::Particles).unwrap(),
                    sprite_number: 0,
                };

                self.per_stroke_particles.push(
                    entities.build_entity()
                        .with(local_transform, &mut trans)
                        .with(local_particle, &mut parts)
                        .with(sprite_render, &mut srs)
                        .build()
                );


                while (cx, cy) != (*px, *py) {
                    //in betweeners
                    
                    let mut local_transform = Transform::default();
                    local_transform.set_translation_xyz(0.0, 0.0, 3.0);
                    
                    let mut local_particle = Particle::new(cx, cy, 0, ParticleDeathType::Ui);
                    local_particle.set_ui(ui_index);

                    let sprite_render = SpriteRender {
                        sprite_sheet: sprites.get(SpriteSheetLabel::Particles).unwrap(),
                        sprite_number: 0,
                    };

                    self.per_stroke_particles.push(
                        entities.build_entity()
                            .with(local_transform, &mut trans)
                            .with(local_particle, &mut parts)
                            .with(sprite_render, &mut srs)
                            .build()
                    );
                    
                    if cx < *px {
                        let mut dx = *px - cx;
                        if dx > MAX_BRUSH_STROKE_DIST {
                            dx = MAX_BRUSH_STROKE_DIST;
                        }
                        cx += dx;
                    }
                    if cx > *px {
                        let mut dx = *px - cx;
                        if dx < -MAX_BRUSH_STROKE_DIST {
                            dx = -MAX_BRUSH_STROKE_DIST;
                        }
                        cx += dx;
                    }

                    if cy < *py {
                        let mut dy = *py - cy;
                        if dy > MAX_BRUSH_STROKE_DIST {
                            dy = MAX_BRUSH_STROKE_DIST;
                        }
                        cy += dy;
                    }
                    if cy > *py {
                        let mut dy = *py - cy;
                        if dy < -MAX_BRUSH_STROKE_DIST {
                            dy = -MAX_BRUSH_STROKE_DIST;
                        }
                        cy += dy;
                    }
                }
                
                lpx = *px;
                lpy = *py;         
            }
        }
        if checked {
            self.time_counter = 0.0;
        }
    }
}
