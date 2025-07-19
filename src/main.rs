use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::SpriteImageMode::Scale;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Resource)]
struct Grid {
    dimension: (usize, usize),
    cell_size: Vec2,
    offset: Vec2,
}

#[derive(Component)]
struct PlayerMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Cartographers"),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, show_cell_selector)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Single<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        PlayerMap,
        Sprite {
            anchor: Anchor::BottomLeft,
            image: asset_server.load("textures/maps/map_a.png"),
            image_mode: Scale(ScalingMode::FitCenter),
            custom_size: Some(window.size()),
            ..default()
        },
        Transform::from_translation(window.size().extend(0.0) / -2.0),
    ));

    let dimension = (11, 11);
    let cell_size = Vec2::new(37.1, 36.7);
    let offset = Vec2::new(48.0, 145.0);
    let size = cell_size * Vec2::new(dimension.0 as f32, dimension.1 as f32);

    commands.insert_resource(Grid {
        dimension,
        cell_size,
        offset,
    });

    commands.spawn((
        Visibility::Hidden,
        Mesh2d(meshes.add(Rectangle::from_size(size))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 1.0))),
        Transform::from_translation(
            (offset + size / 2.0 - window.size() / 2.0).extend(1.0) * Vec3::new(1.0, -1.0, 1.0),
        ),
    ));
}

fn show_cell_selector(
    mut map: Single<&mut Sprite, With<PlayerMap>>,
    window: Single<&Window>,
    grid: Res<Grid>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };
    let rect = Rect::from_corners(
        grid.offset,
        grid.offset + grid.cell_size * Vec2::new(grid.dimension.0 as f32, grid.dimension.1 as f32),
    );
    if rect.contains(position) {
        map.color = Color::srgb(0.0, 1.0, 1.0);
    } else {
        map.color = Color::WHITE;
    }
}
