use crate::ImageLoading;
use crate::SelectedImage;
use crate::{ImageReady, ImageViewport};
use bevy::asset::LoadState;
use bevy::prelude::*;

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
                    commands.entity(entity).remove::<ImageLoading>();
                    commands
                        .entity(entity)
                        .insert(ImageReady(image_loading.0.clone()));
                }
                LoadState::NotLoaded => {
                    error!("Image not loaded");
                }
                LoadState::Loading => {
                    debug!("Image loading");
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

pub fn add_ready_image_to_canvas(
    mut commands: Commands,
    mut viewport: Query<(Entity, &mut ImageNode), With<ImageViewport>>,
    ready_image: Query<(Entity, &ImageReady), Added<ImageReady>>,
) {
    if ready_image.iter().count() == 0 {
        return;
    }

    let (canvas_eid, mut canvas) = match viewport.iter_mut().next() {
        Some((canvas_eid, canvas)) => (canvas_eid, canvas),
        None => {
            error!("Canvas not found");
            return;
        }
    };

    for (ready_image_eid, ready_image) in ready_image.iter() {
        commands.entity(canvas_eid).insert_if_new(SelectedImage);
        canvas.image = ready_image.0.clone();
        commands.entity(ready_image_eid).despawn();
    }
}
