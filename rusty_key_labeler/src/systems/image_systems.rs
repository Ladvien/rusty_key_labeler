use crate::{resources::AppData, settings::MAIN_LAYER, Config, MainCamera, SelectedImage};
use crate::{CanvasMarker, ComputedCanvasViewportData, TopRightPanelUI};
use crate::{
    DebounceTimer, ImageLoading, ImageWithUninitializedScale,
    {CurrentFileNameLabelUpdateNeeded, UiLabelingIndexUpdateNeeded},
};
use bevy::prelude::*;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, ShapeBundle},
};
use yolo_io::ImageLabelPair;

#[warn(clippy::too_many_arguments)]
pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
    debounced_timer: Query<Entity, (With<DebounceTimer>, With<SelectedImage>)>,
    mut main_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
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

    let mut camera = match main_camera.iter_mut().next() {
        Some(camera) => camera,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // We need to reset the camera scale to prepare
    // for centering the next image.
    camera.scale = 1.0;

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

// pub fn center_image_on_load(
//     mut commands: Commands,
//     canvas_data: Query<&ComputedCanvasViewportData>,
//     mut uninitialized_images: Query<
//         (Entity, &Sprite, &mut Transform),
//         With<ImageWithUninitializedScale>,
//     >,
//     mut main_camera: Query<
//         (&mut OrthographicProjection, &mut Transform),
//         (With<MainCamera>, Without<ImageWithUninitializedScale>),
//     >,
//     images: Res<Assets<Image>>,
// ) {
//     if canvas_data.iter().count() == 0 {
//         return;
//     }

//     let canvas_data = match canvas_data.iter().next() {
//         Some(data) => data,
//         None => {
//             error!("Canvas data not found");
//             return;
//         }
//     };

//     let (entity, sprite, mut transform) = match uninitialized_images.iter_mut().next() {
//         Some((entity, sprite, transform)) => (entity, sprite, transform),
//         None => {
//             return;
//         }
//     };

//     transform.translation = canvas_data.translation;

//     info!("Image path: {:?}", sprite.image.path());
//     let image_size = match images.get(&sprite.image) {
//         Some(image) => Vec2::new(image.width() as f32, image.height() as f32),
//         None => {
//             error!("Image not found");
//             return;
//         }
//     };

//     // Adjust camera to fit image.
//     let (mut projection, mut camera_transform) = match main_camera.iter_mut().next() {
//         Some((projection, camera_transform)) => (projection, camera_transform),
//         None => {
//             error!("Main camera not found");
//             return;
//         }
//     };

//     // camera_transform.translation = canvas_data.translation;

//     // let height_scale_factor = image_size.y / (projection.area.height() - canvas_data.y_offset / 2.);
//     // let width_scale_factor = image_size.x / (projection.area.width() - canvas_data.x_offset / 2.);

//     let height_scale_factor = image_size.y / canvas_data.height;
//     let width_scale_factor = image_size.x / canvas_data.width;

//     let mut scale_factor = 1.0;
//     if height_scale_factor > width_scale_factor {
//         scale_factor = height_scale_factor;
//     } else {
//         scale_factor = width_scale_factor;
//     }

//     projection.scale = scale_factor;

//     commands
//         .entity(entity)
//         .remove::<ImageWithUninitializedScale>();
// }

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
        ImageWithUninitializedScale,
        Transform::from_translation(Vec3::new(0., 0., 0.)),
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

    info!("Current file name: {}", current_file_name);
    commands.spawn(CurrentFileNameLabelUpdateNeeded(current_file_name));
}

pub fn compute_canvas_viewport_data(
    mut commands: Commands,
    top_right_hands_panel: Query<&ComputedNode, With<TopRightPanelUI>>,
    mut computed_data: Query<&mut ComputedCanvasViewportData>,
    window: Query<&Window>,
    main_camera: Query<&OrthographicProjection, With<MainCamera>>,
) {
    let top_right_ui_compute_node = match top_right_hands_panel.iter().next() {
        Some(cnode) => cnode,
        None => {
            error!("Top right panel not found");
            return;
        }
    };

    let window = match window.iter().next() {
        Some(window) => window,
        None => {
            error!("Window not found");
            return;
        }
    };

    let main_camera_projection = match main_camera.iter().next() {
        Some(projection) => projection,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // WILO: This calculates size correctly for MAIN_CAMERA.
    let canvas_width = top_right_ui_compute_node.size().x * main_camera_projection.scale / 2.;
    let canvas_height = top_right_ui_compute_node.size().y * main_camera_projection.scale / 2.;

    let unscaled_x_offset =
        window.physical_width() as f32 - top_right_ui_compute_node.size().x - 12. - 16.;
    let x_offset_extent = unscaled_x_offset / 2. * top_right_ui_compute_node.inverse_scale_factor()
        / main_camera_projection.scale;

    let unscaled_y_offset =
        window.physical_height() as f32 - top_right_ui_compute_node.size().y - 12. - 16.;

    // info!("Outline_width: {}",

    let y_offset_extent = unscaled_y_offset / 2. * top_right_ui_compute_node.inverse_scale_factor()
        / main_camera_projection.scale;

    if canvas_width <= 0.0
        || canvas_height <= 0.0
        || x_offset_extent <= 0.0
        || y_offset_extent <= 0.0
    {
        info!("Computed canvas viewport data is unavailable");
        return;
    }

    // If the dimensions haven't changed, then we don't need to update the data.
    if computed_data.iter().count() > 0 {
        let data = match computed_data.iter().next() {
            Some(data) => data,
            None => {
                error!("Computed data not found");
                return;
            }
        };

        if data.width == canvas_width
            && data.height == canvas_height
            && data.translation.x == x_offset_extent
            && data.translation.y == y_offset_extent
        {
            return;
        }
    }

    let data = ComputedCanvasViewportData {
        width: canvas_width,
        height: canvas_height,
        // WILO: I need to figure out how to make the offset equal this value.
        translation: Vec3::new(x_offset_extent, y_offset_extent, 0.0),
        // translation: Vec3::new(126., 37.0, 0.0),
    };

    // 2560x1440
    // 2560 * 0.75 = 1920
    // 1440 * 0.90 = 1296

    info!("Computed canvas viewport data: {:#?}", data);

    if computed_data.iter().count() == 0 {
        commands.spawn(data);
        return;
    }

    for mut computed in computed_data.iter_mut() {
        *computed = data.clone();
    }
}

pub fn image_view_system(
    mut commands: Commands,
    window: Query<&Window>,
    mut selected_image: Query<(Entity, &mut Sprite, &Transform), With<SelectedImage>>,
    mut main_camera: Query<(&Camera, &mut OrthographicProjection), With<MainCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    app_data: Res<AppData>,
    images: Res<Assets<Image>>,
    canvas_data: Query<&ComputedCanvasViewportData, Changed<ComputedCanvasViewportData>>,
    canvas_marker: Query<Entity, With<CanvasMarker>>,
    main_camera_transform: Query<&Transform, With<MainCamera>>,
) {
    if canvas_data.iter().count() == 0 {
        return;
    }

    if canvas_marker.iter().count() > 0 {
        return;
    }

    let canvas_data = match canvas_data.iter().next() {
        Some(data) => data,
        None => {
            error!("Canvas data not found");
            return;
        }
    };

    let (camera, projection) = match main_camera.iter_mut().next() {
        Some((camera, projection)) => (camera, projection),
        None => {
            error!("Main camera not found");
            return;
        }
    };

    let transform = Transform::from_translation(canvas_data.translation);

    // WILO: This calculates size correctly for MAIN_CAMERA.
    let size = Vec2::new(canvas_data.width, canvas_data.height);

    let debug_canvas = (
        Name::new("debug_canvas"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                transform,
                hollow: true,
                // thickness: 5.0,
                render_layers: Some(MAIN_LAYER),
                ..ShapeConfig::default_2d()
            },
            size,
        ),
        MAIN_LAYER,
    );

    commands.spawn(debug_canvas);

    // let zoom_factor = app_data.config.settings.zoom_factor;
    // if keyboard_input.pressed(app_data.config.settings.key_map.zoom_in) {
    //     projection.scale *= zoom_factor;
    // }
    // if keyboard_input.pressed(app_data.config.settings.key_map.zoom_out) {
    //     // If image width extent is greater than the projection width,
    //     // then we can zoom in.
    //     projection.scale /= zoom_factor;
    // }

    // commands.spawn(t);
    // if keyboard_input.pressed(app_data.config.settings.key_map.pan_up) {}
    // if keyboard_input.pressed(app_data.config.settings.key_map.pan_down) {}
    // if keyboard_input.pressed(app_data.config.settings.key_map.pan_left) {}
    // if keyboard_input.pressed(app_data.config.settings.key_map.pan_right) {}
}

pub fn translate_image_system(
    mut query: Query<&mut Transform, With<SelectedImage>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
) {
    for mut transform in query.iter_mut() {
        let mut translation = transform.translation;
        if keyboard_input.pressed(config.settings.key_map.pan_up) {
            translation.y += config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_down) {
            translation.y -= config.settings.pan_factor.y * time.delta_secs();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_left) {
            translation.x -= config.settings.pan_factor.x * time.delta_secs();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_right) {
            translation.x += config.settings.pan_factor.x * time.delta_secs();
        }
        transform.translation = translation;
    }
}
