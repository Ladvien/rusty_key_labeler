use crate::{
    bounding_boxes::{BoundingBox, SelectedBoundingBox},
    resources::AppData,
    FocusViewport, MainCamera, SelectedImage,
};
use bevy::prelude::*;

pub fn translate_image_system(
    mut query: Query<&mut Transform, With<SelectedImage>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_data: Res<AppData>,
) {
    for mut transform in query.iter_mut() {
        let mut translation = transform.translation;
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
        transform.translation = translation;
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
    mut query: Query<(Entity, &GlobalTransform, &Sprite), With<MainCamera>>,
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

        // Checking if any bounding box is selected
        let selected = match selected_bounding_box.iter().next() {
            Some((entity, _, _)) => entity,
            None => {
                // If no bounding box is selected, select the first one
                if let Some((entity, bounding_box)) = bounding_boxes.iter().next() {
                    info!("Selecting first bounding box");
                    info!("Bounding box: {:#?}", bounding_box);
                    commands.entity(entity).insert(FocusViewport {
                        width: bounding_box.width,
                        height: bounding_box.height,
                    });
                    return;
                } else {
                    info!("No bounding boxes to select");
                    return;
                };
            }
        };

        for (eid, bounding_box) in bounding_boxes.iter() {
            info!("Bounding box: {:#?}", bounding_box);
        }

        // for (entity, _, _) in selected_bounding_box.iter() {
        //     commands.entity(entity).remove::<SelectedBoundingBox>();
        // }
    }
}
