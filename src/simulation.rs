use crate::input::MainCamera;
use crate::ui::{GameExitEvent, ResetEvent, SimulationStartEvent, SimulationStopEvent};
use bevy::app::AppExit;
use bevy::{prelude::*, time::FixedTimestep};

const SPRITE_SIZE: f32 = 16.5;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.01, 0.1, 0.001)))
            .insert_resource(MouseWorldPositionDraw(None))
            .insert_resource(MouseWorldPositionErase(None))
            .insert_resource(IsSimulationRunning(false))
            .add_startup_system(setup)
            .add_system(exit_game)
            .add_system(set_simulation)
            .add_system(unset_simulation)
            .add_system(reset_board)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.016))
                    .with_system(set_cursor_world_position.label(CellInteraction::Input))
                    .with_system(
                        cell_interaction
                            .label(CellInteraction::Setting)
                            .after(CellInteraction::Input),
                    ),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.25))
                    .with_system(
                        simulation_step
                            .label(CellInteraction::Simulation)
                            .after(CellInteraction::Setting),
                    ),
            );
    }
}

#[derive(Default)]
struct MouseWorldPositionDraw(Option<(f32, f32)>);

#[derive(Default)]
struct MouseWorldPositionErase(Option<(f32, f32)>);

#[derive(Component)]
struct Cell {
    alive: CellState,
}

enum CellState {
    Alive,
    Dead,
    Empty,
    Sick,
    Murdered,
}

#[derive(Default)]
pub struct SpriteImages {
    empty_cell: Sprite,
    alive_cell: Sprite,
    dead_cell: Sprite,
    sick_cell: Sprite,
    murdered_cell: Sprite,
}

#[derive(Default)]
struct IsSimulationRunning(bool);

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CellInteraction {
    Input,
    Setting,
    Simulation,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    for x in 0..super::GRID_SIZE {
        for y in 0..super::GRID_SIZE {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(16.0)),
                        color: Color::BLACK,
                        ..Default::default()
                    },
                    //texture: asset_server.load("sprites/empty_cell.png"),
                    ..Default::default()
                })
                .insert(Transform {
                    translation: Vec3::new((x as f32) * SPRITE_SIZE, (y as f32) * SPRITE_SIZE, 0.0),
                    ..Default::default()
                })
                .insert(Cell {
                    alive: CellState::Empty,
                });
        }
    }

    commands.insert_resource(SpriteImages {
        empty_cell: Sprite {
            custom_size: Some(Vec2::splat(16.0)),
            color: Color::BLACK,
            ..default()
        },
        alive_cell: Sprite {
            custom_size: Some(Vec2::splat(16.0)),
            color: Color::MIDNIGHT_BLUE,
            ..default()
        },
        dead_cell: Sprite {
            custom_size: Some(Vec2::splat(16.0)),
            color: Color::BLACK,
            ..default()
        },
        sick_cell: Sprite {
            custom_size: Some(Vec2::splat(16.0)),
            color: Color::AQUAMARINE,
            ..default()
        },
        murdered_cell: Sprite {
            custom_size: Some(Vec2::splat(16.0)),
            color: Color::PURPLE,
            ..default()
        },
    });
}

fn set_cursor_world_position(
    windows: Res<Windows>,
    main_camera: Query<(&Transform, &OrthographicProjection), With<MainCamera>>,
    mouse_btn: Res<Input<MouseButton>>,
    mut mouse_world_pos_draw: ResMut<MouseWorldPositionDraw>,
    mut mouse_world_pos_erase: ResMut<MouseWorldPositionErase>,
    is_running: Res<IsSimulationRunning>,
) {
    let window = windows.get_primary().unwrap();
    //if !is_running.0 {
    if let Some(pos) = window.cursor_position() {
        let (transform, proj) = main_camera.single();
        let pos_world = get_mouse_world(pos, transform, window, proj);

        if mouse_btn.pressed(MouseButton::Left) {
            *mouse_world_pos_draw = MouseWorldPositionDraw(Some((pos_world.x, pos_world.y)));
        }
        if mouse_btn.pressed(MouseButton::Right) {
            *mouse_world_pos_erase = MouseWorldPositionErase(Some((pos_world.x, pos_world.y)));
        }
    }
    //}
}

fn get_mouse_world(
    pos: Vec2,
    main_transform: &Transform,
    window: &Window,
    proj: &OrthographicProjection,
) -> Vec3 {
    let center = main_transform.translation.truncate();
    let half_width = (window.width() / 2.0) * proj.scale;
    let half_height = (window.height() / 2.0) * proj.scale;
    let left = center.x - half_width;
    let bottom = center.y - half_height;

    Vec3::new(left + pos.x * proj.scale, bottom + pos.y * proj.scale, 0.0)
}

fn cell_interaction(
    mut cells: Query<(&mut Cell, &mut Sprite, &Transform)>,
    mut mouse_world_pos_draw: ResMut<MouseWorldPositionDraw>,
    mut mouse_world_pos_erase: ResMut<MouseWorldPositionErase>,
    sprite_images: Res<SpriteImages>,
    is_running: Res<IsSimulationRunning>,
) {
    let mouse_draw = mouse_world_pos_draw.0.take();
    let mouse_erase = mouse_world_pos_erase.0.take();
    //if !is_running.0 {
    for (mut cell, mut sprite, transform) in cells.iter_mut() {
        if let Some(mouse_world_pos) = mouse_draw {
            if is_in_cell_bounds(
                (mouse_world_pos.0, mouse_world_pos.1),
                (transform.translation.x, transform.translation.y),
                (8.0, 8.0),
            ) {
                cell.alive = CellState::Alive;
                *sprite = sprite_images.alive_cell.clone();
            }
        }

        if let Some(mouse_world_pos) = mouse_erase {
            if is_in_cell_bounds(
                (mouse_world_pos.0, mouse_world_pos.1),
                (transform.translation.x, transform.translation.y),
                (8.0, 8.0),
            ) {
                cell.alive = CellState::Empty;
                *sprite = sprite_images.empty_cell.clone();
            }
        }
    }
    //}
}

fn is_in_cell_bounds(xy: (f32, f32), center: (f32, f32), dims: (f32, f32)) -> bool {
    xy.0 >= center.0 - dims.0
        && xy.0 < center.0 + dims.0
        && xy.1 >= center.1 - dims.1
        && xy.1 < center.1 + dims.1
}

fn simulation_step(
    mut cells: Query<(&mut Cell, &mut Sprite)>,
    is_running: Res<IsSimulationRunning>,
    sprite_images: Res<SpriteImages>,
) {
    if is_running.0 {
        let mut life_grid: Vec<bool> = Vec::new();
        for (cell, _sprite) in cells.iter_mut() {
            life_grid.push(match cell.alive {
                CellState::Alive => true,
                CellState::Murdered | CellState::Sick | CellState::Dead | CellState::Empty => false,
            });
        }

        for (ind, (mut cell, mut sprite)) in cells.iter_mut().enumerate() {
            let mut neighbour_cnt = 0;
            let x = ind as i32 % crate::GRID_SIZE;
            let y = ind as i32 / crate::GRID_SIZE;

            for xi in (x - 1)..(x + 2) {
                for yi in (y - 1)..(y + 2) {
                    if (xi != x || yi != y)
                        && xi >= 0
                        && xi < crate::GRID_SIZE
                        && yi >= 0
                        && yi < crate::GRID_SIZE
                    {
                        let lin_ind = xi + yi * crate::GRID_SIZE;
                        if life_grid[lin_ind as usize] {
                            neighbour_cnt += 1;
                        }
                    }
                }
            }

            if neighbour_cnt < 2 || neighbour_cnt > 3 {
                match cell.alive {
                    CellState::Alive => {
                        cell.alive = CellState::Sick;
                        *sprite = sprite_images.sick_cell.clone();
                    }
                    CellState::Sick => {
                        cell.alive = CellState::Murdered;
                        *sprite = sprite_images.murdered_cell.clone();
                    }
                    CellState::Murdered => {
                        cell.alive = CellState::Dead;
                        *sprite = sprite_images.dead_cell.clone();
                    }
                    CellState::Dead => {
                        cell.alive = CellState::Empty;
                        *sprite = sprite_images.empty_cell.clone();
                    }
                    _ => {}
                }
            }

            if neighbour_cnt == 3 {
                cell.alive = CellState::Alive;
                *sprite = sprite_images.alive_cell.clone();
            }
        }
    }
}

fn exit_game(mut exit: EventWriter<AppExit>, mut event_reader: EventReader<GameExitEvent>) {
    if event_reader.iter().next().is_some() {
        exit.send(AppExit);
    }
}

fn set_simulation(
    mut event_reader: EventReader<SimulationStartEvent>,
    mut start_sim: ResMut<IsSimulationRunning>,
) {
    if event_reader.iter().next().is_some() {
        start_sim.0 = true;
    }
}

fn unset_simulation(
    mut event_reader: EventReader<SimulationStopEvent>,
    mut start_sim: ResMut<IsSimulationRunning>,
) {
    if event_reader.iter().next().is_some() {
        start_sim.0 = false;
    }
}
fn reset_board(
    mut event_reader: EventReader<ResetEvent>,
    mut cell_cq: Query<(&mut Cell, &mut Sprite)>,
    sprite_images: Res<SpriteImages>,
) {
    if event_reader.iter().next().is_some() {
        for (mut cell, mut sprite) in cell_cq.iter_mut() {
            cell.alive = CellState::Empty;
            *sprite = sprite_images.empty_cell.clone();
        }
    }
}
