use crate::asset_manager::{PlayerMaps, TerrainImages};
use crate::terrain::Terrain;
use crate::{AppState, SelectedChoice, SnapSelectedChoiceToCell, WorldPosition};
use bevy::prelude::*;
use std::collections::HashSet;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(
            Update,
            (snap_selected_choice_to_cell, highlight_selected_choice)
                .chain()
                .run_if(on_event::<SnapSelectedChoiceToCell>),
        );
}

#[derive(Clone, Component, Debug)]
pub struct PlayerMap;

#[derive(Clone, Debug, Resource)]
pub struct Grid {
    pub cell_size: Vec2,
    dimension: (usize, usize),
    top_left_cell_offset: Vec2,
    scale: f32,
}

#[derive(Clone, Component, Debug)]
pub struct Cell {
    terrain: Terrain,
    index: (usize, usize),
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

pub fn is_inside_grid(
    grid: Option<Res<Grid>>,
    world_position: Res<WorldPosition>,
    player_map: Single<&Transform, With<PlayerMap>>,
) -> bool {
    let Some(grid) = grid.as_ref() else {
        return false;
    };
    let min = player_map.translation.truncate()
        + (grid.top_left_cell_offset - grid.cell_size.inverse_y() / 2.0) * grid.scale;
    let max = min + (grid.dimension.to_vec2() * grid.cell_size).inverse_y() * grid.scale;
    let rect = Rect::from_corners(min, max);
    rect.contains(**world_position)
}

pub fn snap_selected_choice_to_cell(
    grid: Res<Grid>,
    cells: Query<&Cell>,
    mut event_reader: EventReader<SnapSelectedChoiceToCell>,
    mut selected_choice: Single<(&mut Transform, &mut SelectedChoice)>,
) {
    let cell = event_reader.read().next().expect("cell");
    selected_choice.1.latest_hovered_cell = Some(cell.0);

    let rotation = selected_choice.1.rotation.to_radians();
    let cos = rotation.cos().round();
    let sin = rotation.sin().round();
    let rotation_factor = Vec2::new(cos - sin, cos + sin);

    let cell = cells.get(cell.0).expect("cell");
    let choice_size = selected_choice.1.choice.size(grid.cell_size);
    let reference_cell = (((choice_size / grid.cell_size).yx() - Vec2::X) / 2.0).floor();
    let mut reference_cell_offset =
        (reference_cell - ((choice_size / grid.cell_size).yx() - Vec2::ONE) / 2.0) * grid.cell_size;
    let reference_cell = (reference_cell.x as usize, reference_cell.y as usize);

    if cos == 0.0 {
        reference_cell_offset = reference_cell_offset.yx();
    }
    selected_choice.0.translation = (grid.top_left_cell_offset
        - reference_cell_offset.yx() * rotation_factor
        + (cell.index.1, cell.index.0).to_vec2() * grid.cell_size.inverse_y())
    .extend(selected_choice.0.translation.z);

    let shifted_tiles = selected_choice
        .1
        .choice
        .tiles
        .iter()
        .map(|(row, column)| {
            let shifted = (
                *row as isize - reference_cell.0 as isize,
                *column as isize - reference_cell.1 as isize,
            );
            if rotation.to_degrees() == 90.0 {
                (shifted.1, -shifted.0)
            } else if rotation.to_degrees() == 180.0 {
                (-shifted.0, -shifted.1)
            } else if rotation.to_degrees() == 270.0 {
                (-shifted.1, shifted.0)
            } else {
                shifted
            }
        })
        .collect::<Vec<_>>();

    let occupied_tiles = shifted_tiles
        .into_iter()
        .map(|(row, column)| (-row + cell.index.0 as isize, column + cell.index.1 as isize))
        .collect::<Vec<_>>();

    selected_choice.1.occupied_tiles = Some(occupied_tiles);

    event_reader.clear();
}

fn trigger_grid_snapping(trigger: Trigger<Pointer<Over>>, mut commands: Commands) {
    commands.send_event(SnapSelectedChoiceToCell(trigger.target()));
}

fn setup(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    player_maps: Res<PlayerMaps>,
    window: Single<&Window>,
    terrain_images: Res<TerrainImages>,
) {
    let map_image = images.get(player_maps.side_a.id()).expect("player map");
    let map_size = map_image.size_f32();
    let map_dimension = (11, 11);
    let map_scale = window.height() / map_size.y;
    let map_pos = Vec2::new((map_size.x * map_scale - window.width()) / 2.0, 0.0);

    let map_area = player_maps.side_a_rect;
    let cell_size = map_area.size() / map_dimension.to_vec2();
    let top_left_cell_offset = (map_area.min + (cell_size - map_size) / 2.0).inverse_y();

    commands.insert_resource(Grid {
        dimension: map_dimension,
        cell_size,
        top_left_cell_offset,
        scale: map_scale,
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
    let mut observer = Observer::new(trigger_grid_snapping);
    for column in 0..map_dimension.0 {
        for row in 0..map_dimension.1 {
            let index = (row, column);
            let terrain = if mountains.contains(&index) {
                Terrain::Mountain
            } else {
                Terrain::default()
            };
            let cell_entity = commands
                .spawn((
                    Sprite {
                        image: terrain_images[&terrain].clone(),
                        custom_size: Some(cell_size),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                    Transform::from_translation(
                        (top_left_cell_offset + cell_size * (column, row).to_vec2().inverse_y())
                            .extend(1.0),
                    ),
                    Cell { terrain, index },
                ))
                .id();
            observer.watch_entity(cell_entity);
            commands.entity(map_entity).add_child(cell_entity);
        }
    }
    commands.spawn(observer);
}

fn highlight_selected_choice(
    selected_choice: Single<(&mut Sprite, &SelectedChoice)>,
    cells: Query<&Cell>,
    grid: Res<Grid>,
) {
    let (mut sprite, selected_choice) = selected_choice.into_inner();
    sprite.color = Color::WHITE;
    let Some(occupied_tiles) = selected_choice.occupied_tiles.as_ref() else {
        return;
    };

    let outside_grid = occupied_tiles.iter().cloned().any(|(row, column)| {
        row < 0
            || column < 0
            || row >= grid.dimension.0 as isize
            || column >= grid.dimension.1 as isize
    });

    let placed_cells = cells
        .iter()
        .filter(|cell| cell.terrain != Terrain::None)
        .map(|cell| (cell.index.0 as isize, cell.index.1 as isize))
        .collect::<HashSet<_>>();

    let colliding_with_cell = occupied_tiles
        .iter()
        .any(|tile| placed_cells.contains(tile));

    if outside_grid || colliding_with_cell {
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.5);
    }
}
