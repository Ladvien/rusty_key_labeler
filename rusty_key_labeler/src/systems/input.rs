use std::ops::AddAssign;

use crate::{
    bounding_boxes::{BoundingBox, SelectedBoundingBox},
    resources::AppData,
    DebounceNextImage, DebounceTimer, FocusViewport, ImageLoading, MainCamera, SelectedImage,
    TopRightPanelUI, UITopPanel,
};
use bevy::{math::VectorSpace, prelude::*};
use bevy_inspector_egui::bevy_egui::node;

#[warn(clippy::too_many_arguments)]
pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    debounced_timer: Query<(Entity, &mut DebounceTimer)>,
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

    // Load next image
    let next_image_path = match valid_pairs[app_data.index as usize].image_path.clone() {
        Some(image) => image,
        _ => {
            error!("Image path not found");
            return;
        }
    };
    let next_image = next_image_path.as_path().to_string_lossy().into_owned();

    // Add image to the scene
    let next_image_handle = asset_server.load::<Image>(next_image.clone());

    commands.spawn(ImageLoading(next_image_handle.clone()));
    commands.spawn((
        Name::from("DebounceTimer"),
        DebounceTimer {
            timer: Timer::from_seconds(
                app_data.config.settings.delay_between_images,
                TimerMode::Once,
            ),
        },
    ));
}

pub fn translate_image_system(
    mut canvas: Query<&mut Node, With<SelectedImage>>,
    top_panel: Query<&mut ComputedNode, (With<UITopPanel>, Without<SelectedImage>)>,
    window: Query<&Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_data: Res<AppData>,
) {
    let canvas_container_node = match top_panel.iter().next() {
        Some(node) => node,
        None => return,
    };

    let mut canvas_node = match canvas.iter_mut().next() {
        Some(node) => node,
        None => return,
    };

    let window = match window.iter().next() {
        Some(window) => window,
        None => return,
    };

    let viewport_size = Vec2::new(
        window.physical_height() as f32,
        window.physical_width() as f32,
    );

    let canvas_top_px = canvas_node
        .top
        .resolve(canvas_container_node.size().y, viewport_size)
        .expect("Failed to resolve canvas top");

    let canvas_left_px = canvas_node
        .left
        .resolve(canvas_container_node.size().x, viewport_size)
        .expect("Failed to resolve canvas left");

    if keyboard_input.pressed(app_data.config.settings.key_map.pan_up) {
        let new_top = canvas_top_px + app_data.config.settings.pan_factor.y * time.delta_secs();
        canvas_node.top = Val::Px(new_top);
    }
    if keyboard_input.pressed(app_data.config.settings.key_map.pan_down) {
        let new_top = canvas_top_px - app_data.config.settings.pan_factor.y * time.delta_secs();
        canvas_node.top = Val::Px(new_top);
    }
    if keyboard_input.pressed(app_data.config.settings.key_map.pan_left) {
        let new_left = canvas_left_px + app_data.config.settings.pan_factor.x * time.delta_secs();
        canvas_node.left = Val::Px(new_left);
    }
    if keyboard_input.pressed(app_data.config.settings.key_map.pan_right) {
        let new_left = canvas_left_px - app_data.config.settings.pan_factor.x * time.delta_secs();
        canvas_node.left = Val::Px(new_left);
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
