use bevy::prelude::*;
use input::*;
use simulation::*;
use ui::*;

mod input;
mod simulation;
mod ui;

const GRID_SIZE: i32 = 140;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1024f32,
            height: 720f32,
            resizable: false,
            title: "Game Of Life".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(SimulationPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(MainMenuPlugin)
        .run();
}
