#![allow(dead_code)]

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::SpriteImageMode::Scale;
use bevy_framepace::FramepacePlugin;
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

#[derive(Component)]
struct Cell {
    category: Category,
    index: (usize, usize),
}

#[derive(Default)]
enum Category {
    Field,
    Forest,
    Monster,
    Mountain,
    Village,
    Water,
    #[default]
    None,
}

#[derive(Component)]
struct RowSelector;

#[derive(Component)]
struct ColumnSelector;

impl Category {
    fn get_file_path(&self) -> &str {
        match self {
            Category::Field => "textures/categories/field.png",
            Category::Forest => "textures/categories/forest.png",
            Category::Monster => "textures/categories/monster.png",
            Category::Mountain => "textures/categories/mountain.png",
            Category::Village => "textures/categories/village.png",
            Category::Water => "textures/categories/water.png",
            Category::None => "textures/categories/none.png",
        }
    }
}

#[derive(Component)]
struct Deck(Vec<Entity>);

#[derive(Component)]
struct DiscardPile(Vec<Entity>);

#[derive(Component)]
struct Card;

#[derive(Resource)]
struct CardBacks {
    exploration: Handle<Image>,
    season: Handle<Image>,
    beach: Handle<Image>,
    house: Handle<Image>,
    shape: Handle<Image>,
    tree: Handle<Image>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Cartographers"),
                    ..default()
                }),
                ..default()
            }),
            FramepacePlugin,
            MeshPickingPlugin,
        ))
        .insert_resource(SpritePickingSettings {
            require_markers: false,
            picking_mode: SpritePickingMode::BoundingBox,
        })
        .insert_resource(MeshPickingSettings {
            require_markers: false,
            ray_cast_visibility: RayCastVisibility::Any,
        })
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_systems(Startup, setup)
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
    let cell_size = Vec2::new(37.2, 36.7);
    let offset = Vec2::new(48.0, 145.0);
    let size = cell_size * Vec2::new(dimension.0 as f32, dimension.1 as f32);

    commands.insert_resource(Grid {
        dimension,
        cell_size,
        offset,
    });

    commands
        .spawn((
            Visibility::Hidden,
            Pickable {
                should_block_lower: false,
                is_hoverable: true,
            },
            Mesh2d(meshes.add(Rectangle::from_size(size))),
            Transform::from_translation(
                (offset + size / 2.0 - window.size() / 2.0).extend(1.0) * Vec3::new(1.0, -1.0, 1.0),
            ),
        ))
        .observe(
            |_: Trigger<Pointer<Out>>,
             mut row_selector: Single<
                &mut Visibility,
                (With<RowSelector>, Without<ColumnSelector>),
            >,
             mut column_selector: Single<
                &mut Visibility,
                (With<ColumnSelector>, Without<RowSelector>),
            >| {
                **row_selector = Visibility::Hidden;
                **column_selector = Visibility::Hidden;
            },
        );

    let row_size = Vec2::new(size.x, cell_size.y);
    commands.spawn((
        RowSelector,
        Pickable::IGNORE,
        Visibility::Hidden,
        Mesh2d(meshes.add(Rectangle::from_size(row_size))),
        MeshMaterial2d(materials.add(Color::srgba(0.0, 0.0, 0.0, 0.2))),
        Transform::from_translation(
            (offset + row_size / 2.0 - window.size() / 2.0).extend(1.0) * Vec3::new(1.0, -1.0, 1.0),
        ),
    ));

    let column_size = Vec2::new(cell_size.x, size.y);
    commands.spawn((
        ColumnSelector,
        Pickable::IGNORE,
        Visibility::Hidden,
        Mesh2d(meshes.add(Rectangle::from_size(column_size))),
        MeshMaterial2d(materials.add(Color::srgba(0.0, 0.0, 0.0, 0.2))),
        Transform::from_translation(
            (offset + column_size / 2.0 - window.size() / 2.0).extend(1.0)
                * Vec3::new(1.0, -1.0, 1.0),
        ),
    ));

    let mountains = vec![(1, 3), (2, 8), (5, 5), (8, 2), (9, 7)];
    let mut observer = Observer::new(position_selectors);

    for row in 0..dimension.0 {
        for column in 0..dimension.1 {
            let index = (row, column);
            let cell = Cell {
                category: if mountains.contains(&index) {
                    Category::Mountain
                } else {
                    Category::default()
                },
                index,
            };
            let cell = commands.spawn((
                Sprite {
                    image: asset_server.load(cell.category.get_file_path()),
                    custom_size: Some(cell_size),
                    ..default()
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
                Transform::from_translation(
                    (offset - window.size() / 2.0
                        + cell_size / 2.0
                        + cell_size * Vec2::new(column as f32, row as f32))
                    .extend(1.0)
                        * Vec3::new(1.0, -1.0, 1.0),
                ),
                cell,
            ));
            observer.watch_entity(cell.id());
        }
    }
    commands.spawn(observer);

    let mut cards = (1..=4)
        .map(|i| format!("textures/cards/ambushes/card_{i:02}.png"))
        .collect::<Vec<_>>();
    cards.extend((5..=17).map(|i| format!("textures/cards/explorations/card_{i:02}.png")));

    let mut exploration_cards = Vec::new();
    for card in cards {
        let exploration_card = commands.spawn((Card, Sprite::from_image(asset_server.load(card))));
        exploration_cards.push(exploration_card.id());
    }

    commands.spawn(Deck(exploration_cards));
    commands.spawn(DiscardPile(Vec::new()));

    commands.insert_resource(CardBacks {
        exploration: asset_server.load("textures/cards/explorations/back_exploration.png"),
        season: asset_server.load("textures/cards/seasons/back_season.png"),
        beach: asset_server.load("textures/cards/tasks/beaches/back_beach.png"),
        house: asset_server.load("textures/cards/tasks/houses/back_house.png"),
        shape: asset_server.load("textures/cards/tasks/shapes/back_shape.png"),
        tree: asset_server.load("textures/cards/tasks/trees/back_tree.png"),
    });
}

fn position_selectors(
    trigger: Trigger<Pointer<Over>>,
    query: Query<&Cell>,
    grid: Res<Grid>,
    window: Single<&Window>,
    mut row_selector: Single<
        (&mut Transform, &mut Visibility),
        (With<RowSelector>, Without<ColumnSelector>),
    >,
    mut column_selector: Single<
        (&mut Transform, &mut Visibility),
        (With<ColumnSelector>, Without<RowSelector>),
    >,
) {
    let cell = query.get(trigger.target()).unwrap();
    row_selector.0.translation = Vec3::new(
        row_selector.0.translation.x,
        (grid.offset.y + cell.index.0 as f32 * grid.cell_size.y - window.height() / 2.0
            + grid.cell_size.y / 2.0)
            * -1.0,
        row_selector.0.translation.z,
    );
    column_selector.0.translation = Vec3::new(
        grid.offset.x + cell.index.1 as f32 * grid.cell_size.x - window.width() / 2.0
            + grid.cell_size.x / 2.0,
        column_selector.0.translation.y,
        column_selector.0.translation.z,
    );

    *row_selector.1 = Visibility::Inherited;
    *column_selector.1 = Visibility::Inherited;
}
