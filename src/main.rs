#![allow(dead_code)]

mod asset_manager;
mod cards;
mod deck;
mod map;
mod resource_tracking;
mod terrain;

use crate::asset_manager::{CardBacks, CardFronts, Choices};
use crate::cards::DrawableCard;
use crate::cards::{Card, Scoring};
use crate::map::{
    Grid, PlayerMap, SelectedChoicePlaced, is_inside_grid, snap_selected_choice_to_cell,
};
use crate::terrain::Choice;
use bevy::ecs::relationship::OrderedRelationshipSourceCollection;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_framepace::FramepacePlugin;
use rand::rng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, Default, Deref, Resource)]
struct WorldPosition(Vec2);

#[derive(Component)]
struct MainCamera;

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
struct ChoiceUI;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

// TODO: refactor in separate module
#[derive(Component)]
struct SelectedChoice {
    choice: Choice,
    rotation: f32,
    valid_to_place: bool,
    occupied_tiles: Option<Vec<(isize, isize)>>,
    latest_hovered_cell: Option<Entity>,
}

#[derive(Event)]
struct SnapSelectedChoiceToCell(Entity);

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
            map::plugin,
        ))
        .insert_resource(SpritePickingSettings {
            require_markers: false,
            picking_mode: SpritePickingMode::BoundingBox,
        })
        .insert_resource(MeshPickingSettings {
            require_markers: false,
            ray_cast_visibility: RayCastVisibility::Any,
        })
        .add_event::<SnapSelectedChoiceToCell>()
        .insert_resource(WorldPosition::default())
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::InGame), (setup, spawn_random_tasks))
        .add_systems(PreUpdate, set_world_position)
        .add_systems(
            Update,
            (
                spawn_random_tasks.run_if(input_just_pressed(KeyCode::Enter)),
                draw_card.run_if(
                    input_just_pressed(KeyCode::Space).or(on_event::<SelectedChoicePlaced>),
                ),
                position_selected_choice
                    .after(interactions)
                    .after(snap_selected_choice_to_cell)
                    .run_if(not(is_inside_grid)),
                rotate_selected_choice.before(snap_selected_choice_to_cell),
                create_choices,
                interactions,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

fn setup(mut commands: Commands, card_fronts: Res<CardFronts>, card_backs: Res<CardBacks>) {
    commands.spawn((Camera2d, MainCamera));

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
    choice_ui: Option<Single<Entity, With<ChoiceUI>>>,
    selected_choice: Option<Single<Entity, With<SelectedChoice>>>,
    grid: Res<Grid>,
) {
    if !drawn_card.is_changed() {
        return;
    }
    choice_ui.map(|ui| commands.entity(*ui).despawn());
    selected_choice.map(|choice| commands.entity(*choice).despawn());

    let drawn_card = cards.get(drawn_card.0).expect("card");
    let choices = &choices[drawn_card];
    if choices.is_empty() {
        return;
    }

    commands
        .spawn((
            ChoiceUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Percent(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
        ))
        .with_children(|parent| {
            choices.iter().for_each(|choice| {
                let size = choice.size(grid.cell_size);
                parent.spawn((
                    Node {
                        border: UiRect::all(Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    choice.clone(),
                    BorderRadius::all(Val::Px(8.0)),
                    BorderColor(Color::srgb_u8(10, 10, 10)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    children![(
                        Node {
                            width: Val::Px(size.x),
                            height: Val::Px(size.y),
                            ..default()
                        },
                        ImageNode {
                            image: choice.image.clone(),
                            ..default()
                        }
                    )],
                ));
            });
        });
}

// TODO: refactor to use Observables instead?
fn interactions(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &Choice, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    choice_ui: Single<Entity, With<ChoiceUI>>,
    player_map: Single<Entity, With<PlayerMap>>,
    grid: Res<Grid>,
) {
    for (interaction, choice, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                commands.entity(*choice_ui).despawn();
                commands.entity(*player_map).with_child((
                    SelectedChoice {
                        choice: choice.clone(),
                        rotation: 0.0,
                        valid_to_place: false,
                        occupied_tiles: None,
                        latest_hovered_cell: None,
                    },
                    Sprite {
                        image: choice.image.clone(),
                        custom_size: Some(choice.size(grid.cell_size)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::default().with_z(8.0)),
                ));
            }
            Interaction::Hovered => {
                color.0 = Color::srgb_u8(150, 150, 150);
            }
            Interaction::None => {
                color.0 = Color::srgb_u8(10, 10, 10);
            }
        }
    }
}

fn set_world_position(
    mut world_position: ResMut<WorldPosition>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = *camera;
    if let Some(projected) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        world_position.0 = projected;
    }
}

fn position_selected_choice(
    mut selected_choice: Single<(&mut Transform, &mut SelectedChoice, &mut Sprite)>,
    world_position: Res<WorldPosition>,
    player_map: Single<&Transform, (With<PlayerMap>, Without<SelectedChoice>)>,
) {
    selected_choice.0.translation.x =
        (world_position.x - player_map.translation.x) / player_map.scale.x;
    selected_choice.0.translation.y =
        (world_position.y - player_map.translation.y) / player_map.scale.y;
    selected_choice.1.latest_hovered_cell = None;
    selected_choice.1.occupied_tiles = None;
    selected_choice.2.color = Color::WHITE;
    selected_choice.1.valid_to_place = false;
}

fn rotate_selected_choice(
    mut commands: Commands,
    mut selected_choice: Single<(&mut Transform, &mut SelectedChoice)>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_events.read() {
        selected_choice.1.rotation =
            (selected_choice.1.rotation + 90.0 * event.y.signum() + 360.0) % 360.0;
        selected_choice.0.rotation = Quat::from_rotation_z(selected_choice.1.rotation.to_radians());
        selected_choice
            .1
            .latest_hovered_cell
            .map(|cell| commands.send_event(SnapSelectedChoiceToCell(cell)));
    }
}
