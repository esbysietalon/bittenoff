extern crate amethyst;
#[macro_use]
extern crate num_derive;

extern crate noise;

mod game_state;
mod systems;
mod components;

use game_state::*;

use std::fs;
use std::fs::File;
use serde::Deserialize;
use ron::de::from_str;

use amethyst::{
    Logger,
    GameDataBuilder,
    Application,
    input::{InputBundle, StringBindings},
    core::TransformBundle,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend, 
        RenderingBundle,
    },
    utils::application_root_dir,
    window::{DisplayConfig, MonitorIdent, MonitorsAccess},
};

use amethyst::winit::{Event, EventsLoop, Window, WindowEvent, ControlFlow};


use crate::game_state::Config;

fn main() -> amethyst::Result<()> {
    let app_root = application_root_dir()?;
    let binding_path = app_root.join("config").join("bindings.ron");
    let display_config_path = app_root.join("config").join("display.ron");
    let game_config_path = app_root.join("config").join("globals.ron");

    let contents = fs::read_to_string(display_config_path.to_str().unwrap())
        .expect("Error reading display config file");
    let mut display_config: DisplayConfig = from_str(&contents)
        .expect("Error loading display config file");

    let contents = fs::read_to_string(game_config_path.to_str().unwrap())
        .expect("Error reading game config file");
    let game_config: Config = from_str(&contents)
        .expect("Error loading game config file");
    
    if game_config.fullscreen {
        let events_loop = EventsLoop::new();
        let window = Window::new(&events_loop).unwrap();

        window.hide();

        display_config.fullscreen = Some(MonitorIdent::from_primary(&window));
    }
    Logger::from_config(Default::default())
        .level_for("amethyst_rendy", amethyst::LogLevelFilter::Warn)
        .start();
    
    let input_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(systems::MapSystem, "map_update_system", &[])
        .with(systems::PlayerMoveSystem, "player_move_system", &["input_system"])
        .with(systems::PlayerLocalitySystem, "player_locality_system", &[])
        .with(systems::SimpleIdle, "simple_idle_system", &[])
        .with(systems::PhysicalSystem, "physical_system", &[])
        .with(systems::CounterSystem, "fps_system", &[])
        .with(systems::MoveSystem, "move_system", &[])
        .with(systems::RudderSystem, "rudder_system", &[])
        .with_bundle(
        RenderingBundle::<DefaultBackend>::new()
            // The RenderToWindow plugin provides all the scaffolding for opening a window and drawing on it
            .with_plugin(
                RenderToWindow::from_config(display_config)
                    .with_clear([0.0, 0.0, 0.0, 1.0]),
            )
            // RenderFlat2D plugin is used to render entities with a `SpriteRender` component.
            .with_plugin(RenderFlat2D::default()),
    )?;

    let mut load_state = LoadingState::default();
    load_state.config_path = game_config_path.to_str().unwrap().to_string();

    let mut game = Application::new(app_root, load_state, game_data)?;
    game.run();
    
    Ok(())
}
