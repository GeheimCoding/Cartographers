use bevy::prelude::*;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
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
