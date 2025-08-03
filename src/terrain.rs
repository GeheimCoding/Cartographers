use bevy::prelude::*;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, Eq, Hash, PartialEq)]
pub enum Terrain {
    #[default]
    None,
    Forest,
    Village,
    Farm,
    Water,
    Monster,
    Mountain,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub terrain: Terrain,
    pub image: Handle<Image>,
    pub tiles: Vec<(usize, usize)>,
    pub with_coin: bool,
}

impl Terrain {
    pub fn get_file_path(&self) -> &str {
        match self {
            Terrain::None => "textures/terrain/none.png",
            Terrain::Forest => "textures/terrain/forest.png",
            Terrain::Village => "textures/terrain/village.png",
            Terrain::Farm => "textures/terrain/farm.png",
            Terrain::Water => "textures/terrain/water.png",
            Terrain::Monster => "textures/terrain/monster.png",
            Terrain::Mountain => "textures/terrain/mountain.png",
        }
    }
}
