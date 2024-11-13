use bevy::app::App;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use crossterm::event::KeyEvent;
use shared::player::Player;
use shared::Map;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

#[derive(Component)]
struct Cell {
    row: usize,
    column: usize,
}

#[derive(Resource)]
struct Grid {
    cells: Vec<Vec<Entity>>,
    rows: usize,
    columns: usize,
}

pub async fn render(
    _event_rx: Receiver<KeyEvent>,
    _rx: Receiver<(Map, HashMap<u16, Player>)>,
    _conn_rx: Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Grid {
            cells: Vec::new(),
            rows: 6,
            columns: 10,
        })
        .add_systems(Startup, setup)
        .run();
    Ok(())
}

fn setup(mut commands: Commands, mut grid: ResMut<Grid>) {
    // Set up the 2D camera
    commands.spawn_empty().insert(Camera2dBundle::default());

    let rows = grid.rows;
    let columns = grid.columns;
    let cell_size = 40.0;

    // Initialize the grid cells
    grid.cells = vec![vec![Entity::PLACEHOLDER; columns]; rows];

    for row in 0..rows {
        for col in 0..columns {
            // Calculate the position of each cell
            let x = col as f32 * cell_size - (columns as f32 * cell_size) / 2.0 + cell_size / 2.0;
            let y = row as f32 * cell_size - (rows as f32 * cell_size) / 2.0 + cell_size / 2.0;

            // Spawn a sprite for the cell
            let cell_entity = commands
                .spawn_empty()
                .insert(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.7, 0.7, 0.7),
                        custom_size: Some(Vec2::new(cell_size - 2.0, cell_size - 2.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(x, y, 0.0),
                    ..Default::default()
                })
                .insert(Cell { row, column: col })
                .id();

            grid.cells[row][col] = cell_entity;
        }
    }

    // Optionally, establish neighbor relationships with wrapping
    for row in 0..rows {
        for col in 0..columns {
            let cell_entity = grid.cells[row][col];

            // Calculate wrapped indices for neighbors
            let north_row = (row + rows - 1) % rows;
            let south_row = (row + 1) % rows;
            let west_col = (col + columns - 1) % columns;
            let east_col = (col + 1) % columns;

            let neighbors = vec![
                grid.cells[north_row][col], // North
                grid.cells[south_row][col], // South
                grid.cells[row][west_col],  // West
                grid.cells[row][east_col],  // East
            ];

            // Here you can add components or resources to store the neighbors if needed
            // For now, we're just setting up the structure
        }
    }
}
