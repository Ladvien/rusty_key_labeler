use crate::{
    bounding_boxes::{BoundingBox, SelectedBoundingBox},
    resources::AppData,
    CenterInViewport, DebounceTimer, FocusInViewport, MainCamera, SelectedImage,
};
use bevy::{prelude::*, render::camera};
use itertools::Itertools;

use super::{start_image_load, CornerHandle};

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

    let (mut projection, mut camera_transform) = main_camera.single_mut();

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

pub fn translate_image_system(
    mut main_camera: Query<&mut Transform, With<MainCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_data: Res<AppData>,
) {
    for mut main_camera in main_camera.iter_mut() {
        let mut translation = main_camera.translation;
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_up) {
            translation.y -= app_data.config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_down) {
            translation.y += app_data.config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_left) {
            translation.x += app_data.config.settings.pan_factor.x * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_right) {
            translation.x -= app_data.config.settings.pan_factor.x * time.delta_secs();
        }
        main_camera.translation = translation;
    }
}

pub fn zoom_image_system(
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    app_data: Res<AppData>,
) {
    for mut projection in query.iter_mut() {
        let mut scale = projection.scale;
        let zoom_factor = app_data.config.settings.zoom_factor;
        if keyboard_input.pressed(app_data.config.settings.key_map.zoom_in) {
            scale *= zoom_factor;
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.zoom_out) {
            scale /= zoom_factor;
        }
        projection.scale = scale;
    }
}

pub fn change_bounding_box_selection(
    mut commands: Commands,
    app_data: ResMut<AppData>,
    selected_image: Query<(Entity, &Sprite, &SelectedImage)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bounding_boxes: Query<(Entity, &BoundingBox)>,
    selected_bounding_box: Query<(Entity, &BoundingBox), With<SelectedBoundingBox>>,
    images: Res<Assets<Image>>,
    corner_handles: Query<Entity, With<CornerHandle>>,
) {
    // Behavior
    // Upon tab key press
    // 1. Collect all bounding boxes and sort them by index.
    // 2. Check if any bounding box is selected.
    //      1. If no, select the first bounding box.
    // 3. If a bounding box is selected, select the next bounding box.
    // 4. If last bounding box is selected, clear all selected bounding boxes
    //    and fit the viewport to the image.

    // Change on tab press
    if keyboard_input.just_pressed(app_data.config.settings.key_map.change_selection) {
        for entity in corner_handles.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Collect bounding boxes and sort them by index
        let ordered_bounding_boxes: Vec<(Entity, &BoundingBox)> = bounding_boxes
            .iter()
            .sorted_by_key(|(_, bounding_box)| bounding_box.index)
            .collect();

        // Check if any bounding box is selected.

        if let Some((selected_bb_entity, selected_bounding_box)) =
            selected_bounding_box.iter().next()
        {
            // Increment the index to get the next bounding box
            let next_index = selected_bounding_box.index + 1;
            if next_index >= ordered_bounding_boxes.len() {
                reset_bounding_box_selection(commands, images, selected_bb_entity, selected_image);
                return;
            }

            let new_selected_bounding_box = ordered_bounding_boxes
                .iter()
                .find(|(_, bounding_box)| bounding_box.index == next_index);

            if let Some((new_bb_entity, new_bounding_box)) = new_selected_bounding_box {
                select_initial_bounding_box(
                    commands,
                    new_bb_entity,
                    new_bounding_box,
                    selected_bb_entity,
                );
            } else {
                panic!("No bounding box found with index: {}", next_index);
            }
        } else {
            // Since no bounding box is selected, select the first bounding box
            let first_bounding_box = ordered_bounding_boxes.first();

            match first_bounding_box {
                Some((bounding_box_entity, bounding_box)) => {
                    info!("Selecting first bounding box");
                    commands
                        .entity(*bounding_box_entity)
                        .insert(SelectedBoundingBox)
                        .insert(FocusInViewport {
                            width: bounding_box.width,
                            height: bounding_box.height,
                        })
                        .insert(CenterInViewport);
                }
                None => {
                    info!("No bounding boxes to select");
                }
            };
        }
    }
}

fn select_initial_bounding_box(
    mut commands: Commands,
    new_bb_entity: &Entity,
    new_bounding_box: &BoundingBox,
    selected_bb_entity: Entity,
) {
    // Clear all selected bounding boxes
    commands
        .entity(selected_bb_entity)
        .remove::<SelectedBoundingBox>();

    // Set the new bounding box as selected
    commands
        .entity(*new_bb_entity)
        .insert(SelectedBoundingBox)
        .insert(FocusInViewport {
            width: new_bounding_box.width,
            height: new_bounding_box.height,
        })
        .insert(CenterInViewport);
}

fn reset_bounding_box_selection(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    selected_bb_entity: Entity,
    selected_image: Query<(Entity, &Sprite, &SelectedImage)>,
) {
    // Clear all selected bounding boxes
    commands
        .entity(selected_bb_entity)
        .remove::<SelectedBoundingBox>();

    // Fit the viewport to the image
    let (selected_image_entity, sprite, _) = selected_image.single();

    let image = images.get(&sprite.image).unwrap();

    commands
        .entity(selected_image_entity)
        .insert(FocusInViewport {
            width: image.width() as f32,
            height: image.height() as f32,
        });

    commands
        .entity(selected_image_entity)
        .insert(CenterInViewport);
}
