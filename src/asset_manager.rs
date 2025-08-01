use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load_assets);
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

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let card_backs = CardBacks {
        exploration: asset_server.load("textures/cards/explorations/back_exploration.png"),
        season: asset_server.load("textures/cards/seasons/back_season.png"),
        farm: asset_server.load("textures/cards/scoring/farms/back_farm.png"),
        house: asset_server.load("textures/cards/scoring/houses/back_house.png"),
        shape: asset_server.load("textures/cards/scoring/shapes/back_shape.png"),
        tree: asset_server.load("textures/cards/scoring/trees/back_tree.png"),
    };

    commands.insert_resource(card_backs);
}
