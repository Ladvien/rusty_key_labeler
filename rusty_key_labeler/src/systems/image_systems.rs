use crate::{resources::AppData, settings::MAIN_LAYER, MainCamera, SelectedImage};
use crate::{ComputedViewport, FocusViewport, ImageReady};
use crate::{
    DebounceTimer, ImageLoading, {CurrentFileNameLabelUpdateNeeded, UiLabelingIndexUpdateNeeded},
};
use bevy::asset::LoadState;
use bevy::prelude::*;
use yolo_io::ImageLabelPair;

pub fn image_selection_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
    debounced_timer: Query<Entity, (With<DebounceTimer>, With<SelectedImage>)>,
    mut main_camera: Query<(&mut OrthographicProjection, &mut Transform), With<MainCamera>>,
) {
    // Check if debounce timer is still running.
    if debounced_timer.iter().count() > 0 {
        return;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        app_data.index += 1;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
        app_data.index -= 1;
    } else {
        return;
    }

    let valid_pairs = app_data.yolo_project.get_valid_pairs();

    if app_data.index < 0 {
        app_data.index = valid_pairs.len() as isize - 1;
    }

    if app_data.index >= valid_pairs.len() as isize {
        app_data.index = 0;
    }

    let (mut projection, mut camera_transform) = match main_camera.iter_mut().next() {
        Some((projection, camera_transform)) => (projection, camera_transform),
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // We need to reset the camera scale to prepare
    // for centering the next image.
    projection.scale = 1.0;
    camera_transform.translation = Vec3::new(0., 0., 0.);

    let total_images = valid_pairs.len() as isize - 1;
    start_image_load(
        &mut commands,
        asset_server,
        app_data.index,
        total_images,
        app_data.config.settings.delay_between_images,
        valid_pairs,
    );

    // Remove old selected image.
    for entity in query_selected_images.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

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

    let center = Vec3::new(0., 0., 0.);

    commands.spawn((
        Name::new("selected_image"),
        Sprite {
            image: next_image_handle.clone(),
            ..Default::default()
        },
        SelectedImage,
        ImageLoading(next_image_handle),
        Visibility::Hidden,
        Transform::from_translation(center),
        MAIN_LAYER,
        DebounceTimer {
            timer: Timer::from_seconds(delay_between_images, TimerMode::Once),
        },
    ));

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

    commands.spawn(CurrentFileNameLabelUpdateNeeded(current_file_name));
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
                    // TODO: Reset the viewport when window resized.
                    if selected_images.contains(entity) {
                        if let Some(image) = images.get(&image_loading.0) {
                            let width = image.width() as f32;
                            let height = image.height() as f32;

                            commands
                                .entity(entity)
                                .insert(FocusViewport { width, height });
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
