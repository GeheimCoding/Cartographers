use bevy::prelude::*;

#[derive(Clone, Debug)]
pub enum Ambush {
    GoblinAttack01,
    BugbearAssault02,
    KoboldOnslaught03,
    GnollRaid04,
}

#[derive(Clone, Debug)]
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
