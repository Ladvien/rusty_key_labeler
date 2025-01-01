use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use yolo_io::ImageLabelPair;

use crate::utils::{default_hide, default_main_layer};
use crate::SelectedImage;
use crate::{ComputedViewport, FocusInViewport, ImageReady};
use crate::{DebounceTimer, FileNameLabelUpdateNeeded, ImageLoading, UiLabelingIndexUpdateNeeded};

#[derive(Debug, Clone, Component)]
#[require(
    Name,
    Sprite,
    SelectedImage,
    Transform,
    Visibility(default_hide),
    RenderLayers(default_main_layer)
)]
pub struct TargetImage;

pub fn start_image_load(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    index: isize,
    total_images: isize,
    delay_between_images: f32,
    valid_pairs: Vec<ImageLabelPair>,
) {
    debug!("Loading image at index: {}", index);

    // Load next image
    let next_image_path = match valid_pairs[index as usize].image_path.clone() {
        Some(image) => image,
        _ => {
            error!("Image path not found");
            return;
        }
    };
    let next_image = next_image_path.as_path().to_string_lossy().into_owned();

    debug!("Next image: {}", next_image);

    // Add image to the scene
    let next_image_handle = asset_server.load::<Image>(next_image.clone());

    let target_image_eid = commands.spawn(TargetImage).id();

    commands
        .entity(target_image_eid)
        .insert(Name::new("target_image"))
        .insert(Sprite {
            image: next_image_handle.clone(),
            ..Default::default()
        })
        .insert(ImageLoading(next_image_handle.clone()))
        .insert(DebounceTimer {
            timer: Timer::from_seconds(delay_between_images, TimerMode::Once),
        });

    // Update index label
    let index_label = format!("{}/{}", index + 1, total_images + 1);
    commands.spawn(UiLabelingIndexUpdateNeeded(index_label));

    // Update current file name label
    let current_file_name = next_image_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    debug!("Current file name: {}", current_file_name);

    commands.spawn(FileNameLabelUpdateNeeded(current_file_name));
}

pub fn image_state_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images_being_loaded: Query<(Entity, &ImageLoading)>,
    images: Res<Assets<Image>>,
    selected_images: Query<Entity, With<SelectedImage>>,
    viewport: Query<(Entity, &ComputedViewport)>,
) {
    // Check if images are still being loaded
    if viewport.iter().count() == 0 {
        return;
    }

    for (entity, image_loading) in images_being_loaded.iter() {
        match asset_server.get_load_state(&image_loading.0) {
            Some(image_handle) => match image_handle {
                LoadState::Loaded => {
                    info!("Image loaded");

                    // Whenever the SelectedImage is loaded, we should scale it to
                    // fit the viewport.
                    if selected_images.contains(entity) {
                        if let Some(image) = images.get(&image_loading.0) {
                            let width = image.width() as f32;
                            let height = image.height() as f32;

                            commands
                                .entity(entity)
                                .insert(FocusInViewport { width, height });
                        }
                    }

                    commands.entity(entity).remove::<ImageLoading>();
                    commands
                        .entity(entity)
                        .insert(ImageReady(image_loading.0.clone()));

                    // Make the image visible now that it's loaded.
                    commands.entity(entity).insert(Visibility::Visible);
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
