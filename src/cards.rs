use crate::asset_manager::TerrainImages;
use crate::terrain::{Choice, Terrain};
use bevy::image::TextureFormatPixelInfo;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Card {
    DrawableCard(DrawableCard),
    Season(Season),
    Scroll(Scroll),
    Scoring(Scoring),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DrawableCard {
    Ambush(Ambush),
    Exploration(Exploration),
}

impl From<DrawableCard> for Card {
    fn from(card: DrawableCard) -> Self {
        Self::DrawableCard(card)
    }
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

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Season {
    Spring18,
    Summer19,
    Fall20,
    Winter21,
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Scroll {
    ScrollA22,
    ScrollB23,
    ScrollC24,
    ScrollD25,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Scoring {
    Tree(TreeScoring),
    Farm(FarmScoring),
    House(HouseScoring),
    Shape(ShapeScoring),
}

impl From<Scoring> for Card {
    fn from(scoring: Scoring) -> Self {
        Self::Scoring(scoring)
    }
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum TreeScoring {
    SentinelWood26,
    Greenbough27,
    Treetower28,
    StonesideForest29,
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum FarmScoring {
    CanalLake30,
    MagesValley31,
    TheGoldenGranary32,
    ShoresideExpanse33,
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum HouseScoring {
    Wildholds34,
    GreatCity35,
    GreengoldPlains36,
    Shieldgate37,
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum ShapeScoring {
    Borderlands38,
    LostBarony39,
    TheBrokenRoad40,
    TheCauldrons41,
}

impl Card {
    pub fn get_paths() -> Vec<(Self, String)> {
        let mut paths = Vec::new();

        macro_rules! push_paths {
            ($name:ty, $card:expr, $path:literal, $offset:literal) => {
                <$name>::iter().enumerate().for_each(|(i, c)| {
                    paths.push((
                        $card(c).into(),
                        format!("textures/cards/{}/card_{:02}.png", $path, i + $offset),
                    ));
                });
            };
        }
        push_paths!(Ambush, DrawableCard::Ambush, "ambushes", 1);
        push_paths!(Exploration, DrawableCard::Exploration, "explorations", 5);
        push_paths!(Season, Card::Season, "seasons", 18);
        push_paths!(Scroll, Card::Scroll, "scrolls", 22);
        push_paths!(TreeScoring, Scoring::Tree, "scoring/trees", 26);
        push_paths!(FarmScoring, Scoring::Farm, "scoring/farms", 30);
        push_paths!(HouseScoring, Scoring::House, "scoring/houses", 34);
        push_paths!(ShapeScoring, Scoring::Shape, "scoring/shapes", 38);

        paths
    }
}

impl DrawableCard {
    pub fn generate_choices(
        &self,
        images: &Assets<Image>,
        asset_server: &AssetServer,
        terrain_images: &TerrainImages,
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
                let terrain_image = images.get(&terrain_images[terrain]).expect(&format!(
                    "image for {terrain:?} should have been full loaded at this point"
                ));
                choices.push(Choice {
                    terrain: terrain.clone(),
                    image: asset_server.add(generate_choice_image(&tiles, terrain_image)),
                    tiles: tiles.clone(),
                    with_coin,
                });
            }
        }
        choices
    }
}

fn generate_choice_image(tiles: &[(usize, usize)], terrain_image: &Image) -> Image {
    let terrain_size = terrain_image.texture_descriptor.size;
    let (terrain_width, terrain_height) =
        (terrain_size.width as usize, terrain_size.height as usize);
    let total_width = tiles.iter().map(|(_, y)| y + 1).max().expect("tiles") * terrain_width;
    let total_height = tiles.iter().map(|(x, _)| x + 1).max().expect("tiles") * terrain_height;

    let format = terrain_image.texture_descriptor.format;
    let mut choice_image = Image::new_fill(
        Extent3d {
            width: total_width as u32,
            height: total_height as u32,
            ..terrain_size
        },
        terrain_image.texture_descriptor.dimension,
        &[0, 0, 0, 0],
        format,
        terrain_image.asset_usage,
    );
    let terrain_data = terrain_image
        .data
        .as_ref()
        .expect("terrain_image data should be present");
    let choice_data = choice_image
        .data
        .as_mut()
        .expect("choice_image data should be present");

    let pixel_size = format.pixel_size();
    let terrain_row_length = terrain_width * pixel_size;

    for (choice_row, choice_column) in tiles {
        let choice_row = (total_height / terrain_height) - choice_row - 1;
        for terrain_row in 0..terrain_height {
            let terrain_row_start = terrain_row * terrain_row_length;
            let choice_row_start = (choice_row * total_width * terrain_height
                + choice_column * terrain_width
                + terrain_row * total_width)
                * pixel_size;

            choice_data[choice_row_start..choice_row_start + terrain_row_length].copy_from_slice(
                &terrain_data[terrain_row_start..terrain_row_start + terrain_row_length],
            );
        }
    }
    choice_image
}
