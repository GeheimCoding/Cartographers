use crate::cards::{Ambush, DrawableCard, Exploration};
use crate::terrain::{Choice, Terrain};
use bevy::prelude::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use strum::IntoEnumIterator;

pub fn plugin(app: &mut App) {
    app.add_event::<TerrainImagesLoaded>()
        .add_systems(Startup, load_assets)
        .add_systems(
            PreUpdate,
            (
                track_resource::<TerrainImages>
                    .run_if(resource_exists::<ResourceTracker<TerrainImages>>),
                track_resource::<Choices>.run_if(resource_exists::<ResourceTracker<Choices>>),
                generate_choices.run_if(on_event::<TerrainImagesLoaded>),
            ),
        );
}

#[derive(Resource)]
pub struct CardBacks {
    exploration: Handle<Image>,
    season: Handle<Image>,
    farm: Handle<Image>,
    house: Handle<Image>,
    shape: Handle<Image>,
    tree: Handle<Image>,
}

#[derive(Debug, Deref, Resource)]
pub struct TerrainImages(pub HashMap<Terrain, Handle<Image>>);

#[derive(Debug, Deref, Resource)]
pub struct Choices(HashMap<DrawableCard, Vec<Choice>>);

// TODO: extract resource tracking into it's own module?
trait TrackableResource: Resource {
    fn get_handles(&self) -> Vec<UntypedHandle>;
    fn on_fully_loaded(&self, _commands: &mut Commands) {}
}

#[derive(Resource)]
struct ResourceTracker<T: TrackableResource>(PhantomData<T>);

// TODO: refactor with one-shot systems?
#[derive(Event)]
struct TerrainImagesLoaded;

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CardBacks {
        exploration: asset_server.load("textures/cards/explorations/back_exploration.png"),
        season: asset_server.load("textures/cards/seasons/back_season.png"),
        farm: asset_server.load("textures/cards/scoring/farms/back_farm.png"),
        house: asset_server.load("textures/cards/scoring/houses/back_house.png"),
        shape: asset_server.load("textures/cards/scoring/shapes/back_shape.png"),
        tree: asset_server.load("textures/cards/scoring/trees/back_tree.png"),
    });

    commands
        .insert_resource(TerrainImages(HashMap::from_iter(Terrain::iter().map(
            |terrain| (terrain.clone(), asset_server.load(terrain.get_file_path())),
        ))));
    commands.insert_resource(ResourceTracker::<TerrainImages>(PhantomData));
}

impl TrackableResource for TerrainImages {
    fn get_handles(&self) -> Vec<UntypedHandle> {
        self.0
            .values()
            .map(|handle| handle.clone().untyped())
            .collect()
    }

    fn on_fully_loaded(&self, commands: &mut Commands) {
        commands.send_event(TerrainImagesLoaded);
    }
}

impl TrackableResource for Choices {
    fn get_handles(&self) -> Vec<UntypedHandle> {
        self.0
            .values()
            .flat_map(|choices| choices.iter().map(|choice| choice.image.clone().untyped()))
            .collect()
    }

    fn on_fully_loaded(&self, _commands: &mut Commands) {
        // TODO: add states and switch to running state at this point
        info!("Generated all choices!");
    }
}

fn track_resource<T: TrackableResource>(
    mut commands: Commands,
    trackable_resource: Res<T>,
    asset_server: Res<AssetServer>,
) {
    if trackable_resource
        .get_handles()
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle))
    {
        trackable_resource.on_fully_loaded(&mut commands);
        commands.remove_resource::<ResourceTracker<T>>();
    }
}

fn generate_choices(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    terrain_images: Res<TerrainImages>,
) {
    commands.insert_resource(Choices(HashMap::from_iter(
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
    commands.insert_resource(ResourceTracker::<Choices>(PhantomData));
}
