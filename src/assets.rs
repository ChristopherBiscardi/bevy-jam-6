use bevy::{
    asset::UntypedAssetId,
    diagnostic::{
        DiagnosticsStore, FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use iyes_progress::{
    Progress, ProgressPlugin, ProgressReturningSystem,
    ProgressTracker,
};

use crate::AppState;

pub struct AppAssetsPlugin;

impl Plugin for AppAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ProgressPlugin::<AppState>::new()
                .with_state_transition(
                    AppState::AssetLoading,
                    AppState::Next,
                ),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .load_collection::<MiscAssets>(),
        )
        .add_systems(
            Update,
            (
                track_fake_long_task
                    .track_progress::<AppState>(),
                print_progress,
            )
                .chain()
                .run_if(in_state(AppState::AssetLoading))
                .after(LoadingStateSet(
                    AppState::AssetLoading,
                )),
        );
    }
}

#[derive(AssetCollection, Resource)]
struct MiscAssets {
    #[asset(path = "gltf/levels.glb")]
    levels: Handle<Gltf>,
}

fn print_progress(
    progress: Res<ProgressTracker<AppState>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    let progress = progress.get_global_progress();
    if progress.done > *last_done {
        *last_done = progress.done;
        info!(
            "[Frame {}] Changed progress: {:?}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                .unwrap_or(0.),
            progress
        );
    }
}

fn is_recursively_loaded(
    handle: impl Into<UntypedAssetId>,
    asset_server: &AssetServer,
) -> bool {
    asset_server
        .get_recursive_dependency_load_state(handle)
        .map(|state| state.is_loaded())
        .unwrap_or(false)
}

const DURATION_LONG_TASK_IN_SECS: f64 = 0.2;
fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_secs_f64() > DURATION_LONG_TASK_IN_SECS
    {
        info!("Long fake task is completed");
        true.into()
    } else {
        false.into()
    }
}
