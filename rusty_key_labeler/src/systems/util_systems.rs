use crate::{
    ComputedCanvasViewportData, DebounceTimer, ImageLoading, ImageReady, MainCamera,
    TopRightPanelUI, UiCamera,
};
use bevy::{asset::LoadState, prelude::*};

pub fn debounce_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DebounceTimer), With<DebounceTimer>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        commands.entity(entity).remove::<DebounceTimer>();
    }
}

pub fn image_state_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images_being_loaded: Query<(Entity, &ImageLoading)>,
) {
    // Check if images are still being loaded.
    for (entity, image_loading) in images_being_loaded.iter() {
        match asset_server.get_load_state(&image_loading.0) {
            Some(image_handle) => match image_handle {
                LoadState::Loaded => {
                    info!("Image loaded");
                    commands.entity(entity).remove::<ImageLoading>();
                    commands
                        .entity(entity)
                        .insert(ImageReady(image_loading.0.clone()));
                }
                LoadState::NotLoaded => {
                    info!("Image not loaded");
                }
                LoadState::Loading => {
                    info!("Image loading");
                }
                LoadState::Failed(arc) => {
                    error!("Image failed to load: {:?}", arc);
                }
            },
            None => {
                error!("Image handle not found");
                return;
            }
        }
    }
}
