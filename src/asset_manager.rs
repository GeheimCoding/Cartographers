use crate::AppState;
use crate::cards::{Ambush, Card, DrawableCard, Exploration};
use crate::resource_tracking::{ResourceTracking, TrackableResource};
use crate::terrain::{Choice, Terrain};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load_assets);
}

#[derive(Clone, Debug, Deref, Resource)]
pub struct CardFronts(pub HashMap<Card, Handle<Image>>);

#[derive(Clone, Debug, Resource)]
pub struct CardBacks {
    pub exploration: Handle<Image>,
    pub season: Handle<Image>,
    pub farm: Handle<Image>,
    pub house: Handle<Image>,
    pub shape: Handle<Image>,
    pub tree: Handle<Image>,
}

#[derive(Clone, Debug, Deref, Resource)]
pub struct TerrainImages(pub HashMap<Terrain, Handle<Image>>);

#[derive(Clone, Debug, Deref, Resource)]
pub struct Choices(pub HashMap<DrawableCard, Vec<Choice>>);

#[derive(Clone, Debug, Resource)]
pub struct PlayerMaps {
    pub side_a: Handle<Image>,
    pub side_b: Handle<Image>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CardFronts(HashMap::from_iter(
        Card::get_paths()
            .into_iter()
            .map(|(card, path)| (card, asset_server.load(path))),
    )));

    commands.insert_resource(CardBacks {
        exploration: asset_server.load("textures/cards/explorations/back_exploration.png"),
        season: asset_server.load("textures/cards/seasons/back_season.png"),
        farm: asset_server.load("textures/cards/scoring/farms/back_farm.png"),
        house: asset_server.load("textures/cards/scoring/houses/back_house.png"),
        shape: asset_server.load("textures/cards/scoring/shapes/back_shape.png"),
        tree: asset_server.load("textures/cards/scoring/trees/back_tree.png"),
    });

    commands.insert_resource(PlayerMaps {
        side_a: asset_server.load("textures/maps/map_a.png"),
        side_b: asset_server.load("textures/maps/map_b.png"),
    });

    commands.insert_trackable_resource(TerrainImages(HashMap::from_iter(
        Terrain::iter()
            .map(|terrain| (terrain.clone(), asset_server.load(terrain.get_file_path()))),
    )));
}

impl TrackableResource for TerrainImages {
    fn get_handles_to_track(&self) -> Vec<UntypedHandle> {
        self.values()
            .map(|handle| handle.clone().untyped())
            .collect()
    }

    fn on_tracked_handles_fully_loaded(&self) -> impl Command {
        |world: &mut World| world.run_system_once(generate_choices).expect("run once")
    }
}

impl TrackableResource for Choices {
    fn get_handles_to_track(&self) -> Vec<UntypedHandle> {
        self.values()
            .flat_map(|choices| choices.iter().map(|choice| choice.image.clone().untyped()))
            .collect()
    }

    fn on_tracked_handles_fully_loaded(&self) -> impl Command {
        |world: &mut World| {
            let player_maps = world.resource::<PlayerMaps>().clone();
            world.commands().insert_trackable_resource(player_maps);
        }
    }
}

impl TrackableResource for PlayerMaps {
    fn get_handles_to_track(&self) -> Vec<UntypedHandle> {
        vec![self.side_a.clone().untyped(), self.side_b.clone().untyped()]
    }

    fn on_tracked_handles_fully_loaded(&self) -> impl Command {
        |world: &mut World| {
            let mut next_state = world
                .get_resource_mut::<NextState<AppState>>()
                .expect("next state");
            next_state.set(AppState::InGame);
        }
    }
}

fn generate_choices(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    terrain_images: Res<TerrainImages>,
) {
    commands.insert_trackable_resource(Choices(HashMap::from_iter(
        Ambush::iter()
            .map(|ambush| DrawableCard::Ambush(ambush))
            .chain(Exploration::iter().map(|exploration| DrawableCard::Exploration(exploration)))
            .map(|drawable_card| {
                (
                    drawable_card.clone(),
                    drawable_card.generate_choices(&images, &asset_server, &terrain_images),
                )
            }),
    )));
}
