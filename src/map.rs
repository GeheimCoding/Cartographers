use crate::asset_manager::{PlayerMaps, TerrainImages};
use crate::terrain::Terrain;
use crate::{AppState, SelectedChoice};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.insert_resource(Rects {
        map_a: Rect::from_corners(Vec2::new(69.0, 201.0), Vec2::new(634.0, 760.0)),
    })
    .add_systems(OnEnter(AppState::InGame), setup);
}

#[derive(Clone, Component, Debug)]
pub struct PlayerMap;

#[derive(Clone, Debug, Resource)]
struct Rects {
    map_a: Rect,
}

#[derive(Clone, Debug, Resource)]
struct Grid {
    dimension: (usize, usize),
    cell_size: Vec2,
    top_left_offset: Vec2,
}

#[derive(Clone, Component, Debug)]
struct Cell {
    terrain: Terrain,
    index: (usize, usize),
    // TODO: move into own resource? hashmap from terrain to scale
    scale: Vec2,
}

trait ToVec2 {
    fn to_vec2(&self) -> Vec2;
}

trait Inverse {
    fn inverse_y(&self) -> Self;
}

impl ToVec2 for (usize, usize) {
    fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}

impl Inverse for Vec2 {
    fn inverse_y(&self) -> Self {
        Self::new(self.x, self.y * -1.0)
    }
}

fn setup(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    player_maps: Res<PlayerMaps>,
    window: Single<&Window>,
    rects: Res<Rects>,
    terrain_images: Res<TerrainImages>,
) {
    let map_image = images.get(player_maps.side_a.id()).expect("player map");
    let map_size = map_image.size_f32();
    let map_dimension = (11, 11);
    let map_scale = window.height() / map_size.y;
    let map_pos = Vec2::new((map_size.x * map_scale - window.width()) / 2.0, 0.0);

    let map_area = rects.map_a;
    let top_left_offset = map_area.min - map_size / 2.0;
    let cell_size = map_area.size() / map_dimension.to_vec2();

    commands.insert_resource(Grid {
        dimension: map_dimension,
        cell_size,
        top_left_offset,
    });
    let map_entity = commands
        .spawn((
            PlayerMap,
            Sprite::from_image(player_maps.side_a.clone()),
            Transform::from_translation(map_pos.extend(-2.0))
                .with_scale(Vec2::splat(map_scale).extend(1.0)),
        ))
        .id();

    let mountains = vec![(1, 3), (2, 8), (5, 5), (8, 2), (9, 7)];
    let mut observer = Observer::new(snap_selected_choice_to_cell);
    for column in 0..map_dimension.0 {
        for row in 0..map_dimension.1 {
            let index = (row, column);
            let terrain = if mountains.contains(&index) {
                Terrain::Mountain
            } else {
                Terrain::default()
            };
            let image = terrain_images[&terrain].clone();
            let terrain_image = images.get(&image).expect("terrain image");
            let terrain_size = terrain_image.size_f32();
            let cell_scale = cell_size / terrain_size;

            let cell_entity = commands
                .spawn((
                    Sprite {
                        image,
                        custom_size: Some(cell_size),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                    Transform::from_translation(
                        (top_left_offset + cell_size * (column, row).to_vec2() + cell_size / 2.0)
                            .inverse_y()
                            .extend(1.0),
                    ),
                    Cell {
                        terrain,
                        index,
                        scale: cell_scale,
                    },
                ))
                .id();
            observer.watch_entity(cell_entity);
            commands.entity(map_entity).add_child(cell_entity);
        }
    }
    commands.spawn(observer);
}

fn snap_selected_choice_to_cell(
    trigger: Trigger<Pointer<Over>>,
    cells: Query<&Cell>,
    _grid: Res<Grid>,
    mut selected_choice: Option<Single<&mut Transform, With<SelectedChoice>>>,
) {
    let Some(selected_choice) = selected_choice.as_mut() else {
        return;
    };
    let cell = cells.get(trigger.target()).expect("cell");

    // TODO: set correct position also based on rotation
    selected_choice.translation.x = selected_choice.translation.x;
    selected_choice.translation.y = selected_choice.translation.y;
    info!("hovered cell: {:?}", cell);
}
