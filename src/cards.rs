use crate::asset_manager::TerrainTextures;
use crate::terrain::{Choice, Terrain};
use bevy::prelude::*;
use strum::EnumIter;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DrawableCard {
    Ambush(Ambush),
    Exploration(Exploration),
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Ambush {
    GoblinAttack01,
    BugbearAssault02,
    KoboldOnslaught03,
    GnollRaid04,
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Exploration {
    TempleRuins05,
    OutpostRuins06,
    GreatRiver07,
    Farmland08,
    Hamlet09,
    ForgottenForest10,
    HinterlandStream11,
    Homestead12,
    Orchard13,
    TreetopVillage14,
    Marshlands15,
    FishingVillage16,
    RiftLands17,
}

#[derive(Clone, Debug)]
pub enum Season {
    Spring18,
    Summer19,
    Fall20,
    Winter21,
}

#[derive(Clone, Debug)]
pub enum Scoring {
    SentinelWood26,
    Greenbough27,
    Treetower28,
    StonesideForest29,
    CanalLake30,
    MagesValley31,
    TheGoldenGranary32,
    ShoresideExpanse33,
    Wildholds34,
    GreatCity35,
    GreengoldPlains36,
    Shieldgate37,
    Borderlands38,
    LostBarony39,
    TheBrokenRoad40,
    TheCauldrons41,
}

#[derive(Clone, Debug)]
pub enum Scroll {
    A,
    B,
    C,
    D,
}

impl DrawableCard {
    pub fn generate_choices(
        &self,
        images: &Assets<Image>,
        asset_server: &AssetServer,
        terrain_textures: &TerrainTextures,
    ) -> Vec<Choice> {
        use Terrain::*;
        #[derive(Default)]
        struct Permutation {
            terrains: Vec<Terrain>,
            tiles: Vec<(Vec<(usize, usize)>, bool)>,
        }
        let permutation = match self {
            DrawableCard::Ambush(ambush) => match ambush {
                Ambush::GoblinAttack01 => Permutation {
                    terrains: vec![Monster],
                    tiles: vec![(vec![(2, 0), (1, 1), (0, 2)], false)],
                },
                Ambush::BugbearAssault02 => Permutation {
                    terrains: vec![Monster],
                    tiles: vec![(vec![(0, 0), (1, 0), (0, 2), (1, 2)], false)],
                },
                Ambush::KoboldOnslaught03 => Permutation {
                    terrains: vec![Monster],
                    tiles: vec![(vec![(0, 0), (1, 0), (2, 0), (1, 1)], false)],
                },
                Ambush::GnollRaid04 => Permutation {
                    terrains: vec![Monster],
                    tiles: vec![(vec![(0, 0), (1, 0), (2, 0), (0, 1), (2, 1)], false)],
                },
            },
            DrawableCard::Exploration(exploration) => match exploration {
                Exploration::TempleRuins05 | Exploration::OutpostRuins06 => Permutation::default(),
                Exploration::GreatRiver07 => Permutation {
                    terrains: vec![Water],
                    tiles: vec![
                        (vec![(0, 0), (1, 0), (2, 0)], true),
                        (vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)], false),
                    ],
                },
                Exploration::Farmland08 => Permutation {
                    terrains: vec![Farm],
                    tiles: vec![
                        (vec![(0, 0), (1, 0)], true),
                        (vec![(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)], false),
                    ],
                },
                Exploration::Hamlet09 => Permutation {
                    terrains: vec![Village],
                    tiles: vec![
                        (vec![(0, 0), (0, 1), (1, 0)], true),
                        (vec![(0, 0), (0, 1), (1, 0), (1, 1), (1, 2)], false),
                    ],
                },
                Exploration::ForgottenForest10 => Permutation {
                    terrains: vec![Forest],
                    tiles: vec![
                        (vec![(0, 1), (1, 0)], true),
                        (vec![(0, 1), (1, 0), (1, 1), (2, 0)], false),
                    ],
                },
                Exploration::HinterlandStream11 => Permutation {
                    terrains: vec![Farm, Water],
                    tiles: vec![(vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)], false)],
                },
                Exploration::Homestead12 => Permutation {
                    terrains: vec![Village, Farm],
                    tiles: vec![(vec![(0, 0), (1, 0), (2, 0), (1, 1)], false)],
                },
                Exploration::Orchard13 => Permutation {
                    terrains: vec![Forest, Farm],
                    tiles: vec![(vec![(1, 0), (1, 1), (1, 2), (0, 2)], false)],
                },
                Exploration::TreetopVillage14 => Permutation {
                    terrains: vec![Forest, Village],
                    tiles: vec![(vec![(0, 0), (0, 1), (0, 2), (1, 2), (1, 3)], false)],
                },
                Exploration::Marshlands15 => Permutation {
                    terrains: vec![Forest, Water],
                    tiles: vec![(vec![(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)], false)],
                },
                Exploration::FishingVillage16 => Permutation {
                    terrains: vec![Village, Water],
                    tiles: vec![(vec![(0, 0), (0, 1), (0, 2), (0, 3)], false)],
                },
                Exploration::RiftLands17 => Permutation {
                    terrains: vec![Forest, Village, Farm, Water, Monster],
                    tiles: vec![(vec![(0, 0)], false)],
                },
            },
        };
        let mut choices = Vec::new();
        for (tiles, with_coin) in permutation.tiles {
            for terrain in permutation.terrains.iter() {
                let base_texture = images.get(&terrain_textures[terrain]).expect(&format!(
                    "texture for {terrain:?} should have been full loaded at this point"
                ));
                choices.push(Choice {
                    terrain: terrain.clone(),
                    texture: asset_server.add(Self::generate_choice_texture(&tiles, base_texture)),
                    tiles: tiles.clone(),
                    with_coin,
                });
            }
        }
        choices
    }

    fn generate_choice_texture(tiles: &[(usize, usize)], base_texture: &Image) -> Image {
        todo!("get inspired by create_choice_image from main.rs")
    }
}
