use bevy::{asset::LoadState, prelude::*};
use bevy_ui_views::VStackUpdatedItems;
use yolo_io::ImageLabelPair;

use crate::{
    bounding_boxes::{BoundingBoxMarker, BoundingBoxPainter},
    resources::AppData,
    settings::MAIN_LAYER,
    ui::{
        CurrentFileNameLabelUpdateNeeded, UIBottomPanel, UILeftPanel, Ui,
        UiLabelingIndexUpdateNeeded,
    },
    Config, DebounceTimer, ImageData, ImageLoading, ImageReady, MainCamera, SelectedImage,
    TopRightPanelUI,
};

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    asset_server: Res<AssetServer>,
) {
    app_data.index = 0;
    let valid_pairs = app_data.yolo_project.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let image_handle = asset_server.load::<Image>(first_image_path.clone());

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2d::default(),
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        // BloomSettings {
        //     intensity: 0.1,
        //     high_pass_frequency: 0.1,
        //     ..Default::default()
        // },
        MAIN_LAYER,
        MainCamera,
    ));

    start_image_load(
        &mut commands,
        asset_server,
        app_data.index,
        valid_pairs.len() as isize - 1,
        0.0,
        valid_pairs,
    );
}

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

#[warn(clippy::too_many_arguments)]
pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
    debounced_timer: Query<Entity, (With<DebounceTimer>, With<SelectedImage>)>,
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
    info!("Loading image at index: {}", index);

    // Load next image
    let next_image_path = match valid_pairs[index as usize].image_path.clone() {
        Some(image) => image,
        _ => {
            error!("Image path not found");
            return;
        }
    };
    let next_image = next_image_path.as_path().to_string_lossy().into_owned();

    info!("Next image: {}", next_image);

    // Add image to the scene
    let next_image_handle = asset_server.load::<Image>(next_image.clone());
    commands.spawn((
        Name::new("selected_image"),
        Sprite {
            image: next_image_handle.clone(),
            ..Default::default()
        },
        SelectedImage,
        ImageLoading(next_image_handle),
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        MAIN_LAYER,
        ZIndex(-10),
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

    info!("Current file name: {}", current_file_name);
    commands.spawn(CurrentFileNameLabelUpdateNeeded(current_file_name));
}

pub fn bounding_boxes_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<
        (Entity, &Sprite),
        (
            With<SelectedImage>,
            With<ImageReady>,
            Without<BoundingBoxMarker>,
        ),
    >,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
    ui: Res<Ui>,
) {
    if query.iter().count() == 0 {
        return;
    }

    info!("Painting bounding boxes");

    let pair = match app_data.yolo_project.pair_at_index(app_data.index) {
        Some(pair) => pair,
        None => {
            error!("Pair not found");
            return;
        }
    };

    let yolo_file = match pair.label_file {
        Some(file) => file,
        None => {
            error!("Label file not found");
            return;
        }
    };

    let mut children = Vec::new();
    let mut ui_items = Vec::new();

    let (selected_image_eid, sprite) = match query.iter().next() {
        Some((eid, sprite)) => (eid, sprite),
        None => {
            error!("Selected image not found");
            return;
        }
    };

    info!("Selected image: {:?}", sprite.image.id());

    match images.get_mut(&sprite.image) {
        Some(image) => {
            // TODO: Keep an eye on this.
            // TODO: What happens if this fails continually?
            commands
                .entity(selected_image_eid)
                .try_insert(BoundingBoxMarker);

            let image_size = Vec2::new(image.width() as f32, image.height() as f32);

            for (index, entry) in yolo_file.entries.iter().enumerate() {
                //
                info!("Adding bounding box: {}", index);
                let bounding_box = bb_painter.get_box(index, entry, image_size);
                let child_id = commands.spawn(bounding_box).id();
                children.push(child_id);

                let color = bb_painter.get_color(entry.class);

                // TODO: I should preload all the color swatches, giving them a path.
                let image = ui.create_image_from_color(color);
                let image_handle = images.add(image);

                let item = ui.create_bounding_box_entry(
                    &app_data.yolo_project.config.export.class_map[&entry.class],
                    image_handle,
                );

                ui_items.push(item);
            }

            // Add bounding box references to UI
            if let Some(left_panel_eid) = app_data.left_panel_eid {
                info!("Updating left panel");
                commands.spawn(VStackUpdatedItems {
                    items: ui_items.clone(),
                    vstack_eid: left_panel_eid,
                });
            }
        }
        None => {
            error!("Image not found");
            return;
        }
    };
    if !children.is_empty() {
        info!("Adding children to selected image");
        commands.entity(selected_image_eid).add_children(&children);
    }
}

// // WILO: I'd like to stop changing the position of the camera
// // and instead change the position and or scale of the image.
pub fn image_view_system(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<Config>,
    window: Query<&Window>,
    just_selected: Query<(Entity, &ImageData), Added<SelectedImage>>,
) {
    // 1. Ensure the image is maxed height or width according to the viewport size.
    // 2. Center the image on first selected.
    // 3. Allow panning and zooming.

    // WILO: Re-think approach, this isn't working.

    let window = window.iter().next().unwrap();
}

// pub fn translate_image_system(
//     mut query: Query<&mut Transform, With<SelectedImage>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     time: Res<Time>,
//     config: Res<Config>,
// ) {
//     for mut transform in query.iter_mut() {
//         let mut translation = transform.translation;
//         if keyboard_input.pressed(config.settings.key_map.pan_up) {
//             translation.y += config.settings.pan_factor.y * time.delta_secs();
//         }
//         if keyboard_input.pressed(config.settings.key_map.pan_down) {
//             translation.y -= config.settings.pan_factor.y * time.delta_secs();
//         }
//         if keyboard_input.pressed(config.settings.key_map.pan_left) {
//             translation.x -= config.settings.pan_factor.x * time.delta_secs();
//         }
//         if keyboard_input.pressed(config.settings.key_map.pan_right) {
//             translation.x += config.settings.pan_factor.x * time.delta_secs();
//         }
//         transform.translation = translation;
//     }
// }
