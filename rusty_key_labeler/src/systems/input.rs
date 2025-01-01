use crate::{
    bounding_boxes::{BoundingBox, SelectedBoundingBox},
    resources::AppData,
    CenterInViewport, DebounceTimer, FocusInViewport, MainCamera, SelectedImage,
};
use bevy::prelude::*;

use super::start_image_load;

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
            translation.y += app_data.config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_down) {
            translation.y -= app_data.config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_left) {
            translation.x -= app_data.config.settings.pan_factor.x * time.delta_secs();
        }
        if keyboard_input.pressed(app_data.config.settings.key_map.pan_right) {
            translation.x += app_data.config.settings.pan_factor.x * time.delta_secs();
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
    mut app_data: ResMut<AppData>,
    mut main_camera: Query<(Entity, &GlobalTransform, &Sprite), With<MainCamera>>,
    mut selected_image: Query<&mut SelectedImage>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bounding_boxes: Query<(Entity, &BoundingBox)>,
    selected_bounding_box: Query<
        (Entity, &GlobalTransform, &BoundingBox),
        With<SelectedBoundingBox>,
    >,
) {
    // Change on tab press
    if keyboard_input.just_pressed(app_data.config.settings.key_map.change_selection) {
        info!("Changing selection");

        for (index, (bounding_box_entity, bounding_box)) in bounding_boxes.iter().enumerate() {
            if index == 0 {
                info!("Selecting first bounding box");
                commands
                    .entity(bounding_box_entity)
                    .insert(SelectedBoundingBox)
                    .insert(FocusInViewport {
                        width: bounding_box.width,
                        height: bounding_box.height,
                    })
                    .insert(CenterInViewport);
                return;
            }
        }

        // // Checking if any bounding box is selected
        // let selected = match selected_bounding_box.iter().next() {
        //     Some((entity, _, _)) => entity,
        //     None => {
        //         // If no bounding box is selected, select the first one
        //         if let Some((entity, bounding_box)) = bounding_boxes.iter().next() {
        //             info!("Selecting first bounding box");
        //             info!("Bounding box: {:#?}", bounding_box);
        //             commands.entity(entity).insert(FocusInViewport {
        //                 width: bounding_box.width,
        //                 height: bounding_box.height,
        //             });
        //             return;
        //         } else {
        //             info!("No bounding boxes to select");
        //             return;
        //         };
        //     }
        // };

        // for (eid, bounding_box) in bounding_boxes.iter() {
        //     info!("Bounding box: {:#?}", bounding_box);
        // }

        // for (entity, _, _) in selected_bounding_box.iter() {
        //     commands.entity(entity).remove::<SelectedBoundingBox>();
        // }
    }
}
