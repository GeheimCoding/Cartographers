#![allow(dead_code)]

mod asset_manager;
mod cards;
mod deck;
mod resource_tracking;
mod terrain;

use crate::asset_manager::{CardBacks, CardFronts, Choices, PlayerMaps, TerrainImages};
use crate::cards::DrawableCard;
use crate::cards::{Card, Scoring};
use crate::terrain::Terrain;
use bevy::ecs::relationship::OrderedRelationshipSourceCollection;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::SpriteImageMode::Scale;
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::rng;
use rand::seq::SliceRandom;

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
    terrain: Terrain,
    index: (usize, usize),
}

#[derive(Component)]
struct RowSelector;

#[derive(Component)]
struct ColumnSelector;

#[derive(Component)]
struct Deck(Vec<Entity>);

#[derive(Component)]
struct TopOfDeck;

#[derive(Component)]
struct DiscardPile(Vec<Entity>);

#[derive(Component)]
struct BottomOfDiscardPile;

#[derive(Component)]
struct DrawnCard(Entity);

#[derive(Component)]
struct Scroll;

#[derive(Component)]
struct GeneratedSprite;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
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
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::InGame), (setup, spawn_random_tasks))
        .add_systems(
            Update,
            (
                spawn_random_tasks.run_if(input_just_pressed(KeyCode::Enter)),
                draw_card.run_if(input_just_pressed(KeyCode::Space)),
                create_choices,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    window: Single<&Window>,
    player_maps: Res<PlayerMaps>,
    terrain_images: Res<TerrainImages>,
    card_fronts: Res<CardFronts>,
    card_backs: Res<CardBacks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        PlayerMap,
        Sprite {
            anchor: Anchor::BottomLeft,
            image: player_maps.side_a.clone(),
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
                terrain: if mountains.contains(&index) {
                    Terrain::Mountain
                } else {
                    Terrain::default()
                },
                index,
            };
            let cell = commands.spawn((
                Sprite {
                    image: terrain_images[&cell.terrain].clone(),
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

    let mut drawable_cards = card_fronts
        .iter()
        .filter_map(|(card, handle)| match card {
            Card::DrawableCard(drawable_card) => Some((drawable_card, handle)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut deck_cards = Vec::new();
    let deck_position = Vec3::new(540.0, 240.0, 2.0);
    drawable_cards.shuffle(&mut rng());
    for (card, handle) in drawable_cards.iter().skip(1).cloned() {
        let exploration_card = commands.spawn((
            card.clone(),
            Sprite {
                image: handle.clone(),
                custom_size: Some(Vec2::new(150.0, 200.0)),
                ..default()
            },
            Transform::from_translation(deck_position),
        ));
        deck_cards.push(exploration_card.id());
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

    let (first_card, handle) = drawable_cards.first().expect("cards").clone();
    let drawn_card = commands
        .spawn((first_card.clone(), Sprite::from_image(handle.clone())))
        .id();
    commands.spawn(DrawnCard(drawn_card));

    commands.spawn(Deck(deck_cards));
    commands.spawn(DiscardPile(Vec::new()));

    for (index, (_, scroll)) in card_fronts
        .iter()
        .filter(|(card, _)| matches!(card, Card::Scroll(_)))
        .enumerate()
    {
        commands.spawn((
            Scroll,
            Sprite {
                image: scroll.clone(),
                custom_size: Some(Vec2::new(100.0, 133.3)),
                ..default()
            },
            Transform::from_translation(Vec3::new(index as f32 * 110.0 + 240.0, -130.0, 2.0)),
        ));
    }
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
    tasks: Query<Entity, With<Scoring>>,
    card_fronts: Res<CardFronts>,
) {
    for task in tasks.iter() {
        commands.entity(task).despawn();
    }
    let mut scoring_cards = card_fronts
        .iter()
        .filter_map(|(card, handle)| match card {
            Card::Scoring(scoring) => Some((scoring, handle)),
            _ => None,
        })
        .collect::<Vec<_>>();
    scoring_cards.shuffle(&mut rng());

    macro_rules! pick {
        ($match_cond:pat) => {
            scoring_cards
                .iter()
                .find_map(|(card, handle)| {
                    if matches!(card, $match_cond) {
                        Some((card.clone(), handle.clone()))
                    } else {
                        None
                    }
                })
                .expect("scoring card with given condition")
        };
    }

    let mut random_scoring = vec![
        pick!(Scoring::Tree(_)),
        pick!(Scoring::Farm(_)),
        pick!(Scoring::House(_)),
        pick!(Scoring::Shape(_)),
    ];
    random_scoring.shuffle(&mut rng());

    random_scoring
        .into_iter()
        .enumerate()
        .for_each(|(index, (scoring, handle))| {
            commands.spawn((
                scoring.clone(),
                Sprite {
                    image: handle.clone(),
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
    mut cards: Query<(&mut Transform, &mut Sprite), With<DrawableCard>>,
    mut visibility: Query<&mut Visibility, (With<DrawableCard>, Without<TopOfDeck>)>,
    mut top_of_deck: Single<&mut Visibility, (With<TopOfDeck>, Without<DrawableCard>)>,
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

fn create_choices(
    drawn_card: Single<Ref<DrawnCard>>,
    choices: Res<Choices>,
    cards: Query<&DrawableCard>,
    mut commands: Commands,
    generated: Query<Entity, With<GeneratedSprite>>,
) {
    if !drawn_card.is_changed() {
        return;
    }

    for generated in generated {
        commands.entity(generated).despawn();
    }

    let drawn_card = cards.get(drawn_card.0).expect("card");
    let choices = &choices[drawn_card];

    for (index, choice) in choices.iter().enumerate() {
        commands.spawn((
            GeneratedSprite,
            Sprite::from_image(choice.image.clone()),
            Transform::from_translation(Vec3::new(-512.0 + 250.0 * index as f32, -100.0, 3.0)),
        ));
    }
}
