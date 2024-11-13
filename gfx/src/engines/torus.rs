use bevy::app::App;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use crossterm::event::KeyEvent;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
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

    // Create a random number generator with a fixed seed
    let mut rng = StdRng::seed_from_u64(42);

    for row in 0..rows {
        for col in 0..columns {
            // Calculate the position of each cell
            let x = col as f32 * cell_size - (columns as f32 * cell_size) / 2.0 + cell_size / 2.0;
            let y = row as f32 * cell_size - (rows as f32 * cell_size) / 2.0 + cell_size / 2.0;

            // Generate a random color
            let random_color = Color::rgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());

            // Spawn a sprite for the cell
            let cell_entity = commands
                .spawn_empty()
                .insert(SpriteBundle {
                    sprite: Sprite {
                        color: random_color,
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
}
