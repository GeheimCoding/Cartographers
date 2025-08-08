#![allow(dead_code)]

mod asset_manager;
mod cards;
mod deck;
mod resource_tracking;
mod terrain;

use bevy::ecs::relationship::OrderedRelationshipSourceCollection;
use bevy::image::TextureFormatPixelInfo;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::sprite::Anchor;
use bevy::sprite::SpriteImageMode::Scale;
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::seq::SliceRandom;
use rand::{Rng, rng};
use std::collections::HashMap;

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

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
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
struct TopOfDeck;

#[derive(Component)]
struct DiscardPile(Vec<Entity>);

#[derive(Component)]
struct BottomOfDiscardPile;

#[derive(Component)]
struct Card;

#[derive(Component)]
struct DrawnCard(Entity);

#[derive(Component, Debug)]
struct Choices(Vec<Choice>);

#[derive(Debug)]
struct Choice {
    categories: Vec<Category>,
    tiles: Vec<(usize, usize)>,
    with_coin: bool,
}

#[derive(Resource)]
struct CardBacks {
    exploration: Handle<Image>,
    season: Handle<Image>,
    beach: Handle<Image>,
    house: Handle<Image>,
    shape: Handle<Image>,
    tree: Handle<Image>,
}

#[derive(Component)]
struct Scroll;

#[derive(Component)]
struct Task;

#[derive(Resource)]
struct Categories(HashMap<Category, Handle<Image>>);

#[derive(Component)]
struct GeneratedSprite;

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
            resource_tracking::plugin,
            asset_manager::plugin,
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
        .add_systems(Startup, (setup, spawn_random_tasks))
        .add_systems(
            Update,
            (
                spawn_random_tasks.run_if(input_just_pressed(KeyCode::Enter)),
                draw_card.run_if(input_just_pressed(KeyCode::Space)),
                create_choices,
            ),
        )
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

    let categories = Categories(HashMap::from_iter(
        [
            Category::Field,
            Category::Forest,
            Category::Monster,
            Category::Mountain,
            Category::Village,
            Category::Water,
            Category::None,
        ]
        .iter()
        .map(|c| (c.clone(), asset_server.load(c.get_file_path()))),
    ));

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
                    image: categories.0[&cell.category].clone(),
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

    let card_backs = CardBacks {
        exploration: asset_server.load("textures/cards/explorations/back_exploration.png"),
        season: asset_server.load("textures/cards/seasons/back_season.png"),
        beach: asset_server.load("textures/cards/tasks/beaches/back_beach.png"),
        house: asset_server.load("textures/cards/tasks/houses/back_house.png"),
        shape: asset_server.load("textures/cards/tasks/shapes/back_shape.png"),
        tree: asset_server.load("textures/cards/tasks/trees/back_tree.png"),
    };

    let mut exploration_cards = Vec::new();
    let deck_position = Vec3::new(540.0, 240.0, 2.0);
    cards.shuffle(&mut rng());
    for card in cards.iter().skip(1) {
        let mut exploration_card = commands.spawn((
            Card,
            Sprite {
                image: asset_server.load(card),
                custom_size: Some(Vec2::new(150.0, 200.0)),
                ..default()
            },
            Transform::from_translation(deck_position),
        ));
        generate_choices(card).map(|choices| exploration_card.insert(choices));
        exploration_cards.push(exploration_card.id());
    }
    commands.spawn((
        TopOfDeck,
        Sprite {
            image: card_backs.exploration.clone(),
            custom_size: Some(Vec2::new(150.0, 200.0)),
            ..default()
        },
        Transform::from_translation(deck_position.with_z(3.0)),
    ));
    commands.spawn((
        BottomOfDiscardPile,
        Sprite {
            image: card_backs.exploration.clone(),
            custom_size: Some(Vec2::new(150.0, 200.0)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.2),
            ..default()
        },
        Transform::from_translation(deck_position.with_z(1.0).with_x(deck_position.x - 180.0)),
    ));

    let first_card = cards.first().expect("cards");
    let mut drawn_card = commands.spawn((Card, Sprite::from_image(asset_server.load(first_card))));
    generate_choices(first_card).map(|choices| drawn_card.insert(choices));
    let drawn_card = drawn_card.id();
    commands.spawn(DrawnCard(drawn_card));

    commands.spawn(Deck(exploration_cards));
    commands.spawn(DiscardPile(Vec::new()));

    commands.insert_resource(card_backs);
    commands.insert_resource(categories);

    (22..=25)
        .map(|i| format!("textures/cards/scrolls/card_{i:02}.png"))
        .enumerate()
        .for_each(|(index, scroll)| {
            commands.spawn((
                Scroll,
                Sprite {
                    image: asset_server.load(scroll),
                    custom_size: Some(Vec2::new(100.0, 133.3)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(index as f32 * 110.0 + 240.0, -130.0, 2.0)),
            ));
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

fn spawn_random_tasks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tasks: Query<Entity, With<Task>>,
) {
    for task in tasks.iter() {
        commands.entity(task).despawn();
    }
    let random_tree = format!(
        "textures/cards/tasks/trees/card_{:02}.png",
        rng().random_range(26..=29)
    );
    let random_beach = format!(
        "textures/cards/tasks/beaches/card_{:02}.png",
        rng().random_range(30..=33)
    );
    let random_house = format!(
        "textures/cards/tasks/houses/card_{:02}.png",
        rng().random_range(34..=37)
    );
    let random_shape = format!(
        "textures/cards/tasks/shapes/card_{:02}.png",
        rng().random_range(38..=41)
    );
    let mut random_tasks = vec![random_tree, random_beach, random_house, random_shape];
    random_tasks.shuffle(&mut rng());

    random_tasks.iter().enumerate().for_each(|(index, task)| {
        commands.spawn((
            Task,
            Sprite {
                image: asset_server.load(task),
                custom_size: Some(Vec2::new(100.0, 133.3)),
                ..default()
            },
            Transform::from_translation(Vec3::new(index as f32 * 110.0 + 240.0, -270.0, 2.0)),
        ));
    });
}

fn draw_card(
    mut deck: Single<&mut Deck>,
    mut discard_pile: Single<&mut DiscardPile>,
    mut drawn_card: Single<&mut DrawnCard>,
    mut cards: Query<(&mut Transform, &mut Sprite), With<Card>>,
    mut visibility: Query<&mut Visibility, (With<Card>, Without<TopOfDeck>)>,
    mut top_of_deck: Single<&mut Visibility, (With<TopOfDeck>, Without<Card>)>,
) {
    let deck = &mut deck.0;
    if deck.is_empty() {
        *visibility
            .get_mut(*discard_pile.0.last().expect("cards"))
            .expect("visibility") = Visibility::Hidden;
        deck.extend(discard_pile.0.drain(..));
        deck.shuffle(&mut rng());
        info!("shuffled");
        **top_of_deck = Visibility::Inherited;
        return;
    }
    discard_pile.0.push(drawn_card.0);
    drawn_card.0 = deck.pop_front().expect("at least one card left in deck");

    if discard_pile.0.len() > 1 {
        *visibility
            .get_mut(discard_pile.0[discard_pile.0.len() - 2])
            .expect("visibility") = Visibility::Hidden;
    }
    {
        let (mut discard_position, mut discard_sprite) = cards
            .get_mut(*discard_pile.0.last().expect("one card"))
            .expect("card");
        discard_sprite.custom_size = Some(Vec2::new(150.0, 200.0));
        discard_position.translation = Vec3::new(540.0 - 180.0, 240.0, 2.0);
    }

    let (mut drawn_position, mut drawn_sprite) = cards.get_mut(drawn_card.0).expect("card");
    drawn_sprite.custom_size = None;
    drawn_position.translation = Vec3::splat(0.0);
    *visibility.get_mut(drawn_card.0).expect("card") = Visibility::Inherited;

    if deck.is_empty() {
        **top_of_deck = Visibility::Hidden;
    }
}

fn generate_choices(card: &str) -> Option<Choices> {
    match card.split("/").last().expect("path") {
        "card_01.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Monster],
            tiles: vec![(2, 0), (1, 1), (0, 2)],
            with_coin: false,
        }])),
        "card_02.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Monster],
            tiles: vec![(0, 0), (1, 0), (0, 2), (1, 2)],
            with_coin: false,
        }])),
        "card_03.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Monster],
            tiles: vec![(0, 0), (1, 0), (2, 0), (1, 1)],
            with_coin: false,
        }])),
        "card_04.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Monster],
            tiles: vec![(0, 0), (1, 0), (2, 0), (0, 1), (2, 1)],
            with_coin: false,
        }])),
        "card_07.png" => Some(Choices(vec![
            Choice {
                categories: vec![Category::Water],
                tiles: vec![(0, 0), (1, 0), (2, 0)],
                with_coin: true,
            },
            Choice {
                categories: vec![Category::Water],
                tiles: vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
                with_coin: false,
            },
        ])),
        "card_08.png" => Some(Choices(vec![
            Choice {
                categories: vec![Category::Field],
                tiles: vec![(0, 0), (1, 0)],
                with_coin: true,
            },
            Choice {
                categories: vec![Category::Field],
                tiles: vec![(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
                with_coin: false,
            },
        ])),
        "card_09.png" => Some(Choices(vec![
            Choice {
                categories: vec![Category::Village],
                tiles: vec![(0, 0), (0, 1), (1, 0)],
                with_coin: true,
            },
            Choice {
                categories: vec![Category::Village],
                tiles: vec![(0, 0), (0, 1), (1, 0), (1, 1), (1, 2)],
                with_coin: false,
            },
        ])),
        "card_10.png" => Some(Choices(vec![
            Choice {
                categories: vec![Category::Forest],
                tiles: vec![(0, 1), (1, 0)],
                with_coin: true,
            },
            Choice {
                categories: vec![Category::Forest],
                tiles: vec![(0, 1), (1, 0), (1, 1), (2, 0)],
                with_coin: false,
            },
        ])),
        "card_11.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Field, Category::Water],
            tiles: vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
            with_coin: false,
        }])),
        "card_12.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Village, Category::Field],
            tiles: vec![(0, 0), (1, 0), (2, 0), (1, 1)],
            with_coin: false,
        }])),
        "card_13.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Forest, Category::Field],
            tiles: vec![(1, 0), (1, 1), (1, 2), (0, 2)],
            with_coin: false,
        }])),
        "card_14.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Forest, Category::Village],
            tiles: vec![(0, 0), (0, 1), (0, 2), (1, 2), (1, 3)],
            with_coin: false,
        }])),
        "card_15.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Forest, Category::Water],
            tiles: vec![(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)],
            with_coin: false,
        }])),
        "card_16.png" => Some(Choices(vec![Choice {
            categories: vec![Category::Village, Category::Water],
            tiles: vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            with_coin: false,
        }])),
        "card_17.png" => Some(Choices(vec![Choice {
            categories: vec![
                Category::Forest,
                Category::Village,
                Category::Field,
                Category::Water,
                Category::Monster,
            ],
            tiles: vec![(0, 0)],
            with_coin: false,
        }])),
        _ => None,
    }
}

fn create_choices(
    drawn_card: Single<Ref<DrawnCard>>,
    choices: Query<&Choices>,
    mut images: ResMut<Assets<Image>>,
    categories: Res<Categories>,
    mut commands: Commands,
    generated: Query<Entity, With<GeneratedSprite>>,
) {
    if !drawn_card.is_changed() {
        return;
    }

    for generated in generated {
        commands.entity(generated).despawn();
    }

    let Ok(choices) = choices.get(drawn_card.0) else {
        return;
    };
    let mut count = 0;
    for choice in choices.0.iter() {
        for category in choice.categories.iter() {
            let Some(mut image) = images.get_mut(&categories.0[category]) else {
                // TODO: proper asset loading first
                return;
            };
            let created_image = create_choice_image(&mut image, choice.tiles.clone());
            let handle = images.add(created_image);

            commands.spawn((
                GeneratedSprite,
                Sprite::from_image(handle),
                Transform::from_translation(Vec3::new(-512.0 + 250.0 * count as f32, -100.0, 3.0)),
            ));
            count += 1;
        }
    }
}

fn create_choice_image(image: &mut Image, tiles: Vec<(usize, usize)>) -> Image {
    let size = image.texture_descriptor.size;
    let format = image.texture_descriptor.format;
    let width = tiles.iter().map(|(_, y)| *y + 1).max().unwrap() as u32 * size.width;
    let height = tiles.iter().map(|(x, _)| *x + 1).max().unwrap() as u32 * size.height;

    let mut choice_image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: size.depth_or_array_layers,
        },
        image.texture_descriptor.dimension,
        &[0, 0, 0, 0],
        format,
        image.asset_usage,
    );
    let pixel_size = format.pixel_size();
    let data = image.data.as_mut().expect("image data should be present");
    let choice_data = choice_image
        .data
        .as_mut()
        .expect("image data should be present");
    for (row, column) in tiles {
        let row = (height / size.height) as usize - row - 1;
        for h in 0..size.height as usize {
            let start = h * size.width as usize * pixel_size;
            let end = start + size.width as usize * pixel_size;

            let choice_start = (row * width as usize * size.height as usize
                + column * size.width as usize
                + h * width as usize)
                * pixel_size;
            let choice_end = choice_start + size.width as usize * pixel_size;

            choice_data[choice_start..choice_end].copy_from_slice(&data[start..end]);
        }
    }
    choice_image
}
