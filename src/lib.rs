use bevy::prelude::*;

pub mod assets;
pub mod dev;
pub mod movement;
pub mod playing;
pub mod postprocessing;
pub mod terrain_chunking;

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    AssetLoading,
    Next,
    Playing,
}
