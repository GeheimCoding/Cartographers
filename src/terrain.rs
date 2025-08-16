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

#[derive(Clone, Component, Debug)]
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

impl Choice {
    pub fn size(&self, cell_size: Vec2) -> Vec2 {
        let max_row = self
            .tiles
            .iter()
            .map(|(row, _)| *row + 1)
            .max()
            .expect("at least one tile");
        let max_column = self
            .tiles
            .iter()
            .map(|(_, column)| *column + 1)
            .max()
            .expect("at least one tile");
        Vec2::new(
            max_column as f32 * cell_size.x,
            max_row as f32 * cell_size.y,
        )
    }
}
