use crate::{resources::AppData, settings::MAIN_LAYER, Config, MainCamera, SelectedImage};
use crate::{CanvasMarker, ComputedViewport, ImageNotYetCentered, TopRightPanelUI};
use crate::{
    DebounceTimer, ImageLoading, ImageNotYetScaled,
    {CurrentFileNameLabelUpdateNeeded, UiLabelingIndexUpdateNeeded},
};
use bevy::color::palettes::css::{INDIAN_RED, LIMEGREEN};
use bevy::render::view;
use bevy::{app, prelude::*};
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, ShapeBundle},
};
use yolo_io::ImageLabelPair;

pub const MAGIC_NUMBER_UI: f32 = 10.0;

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

    let mut projection = match main_camera.iter_mut().next() {
        Some(camera) => camera,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // We need to reset the camera scale to prepare
    // for centering the next image.
    projection.scale = 1.0;

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

pub fn center_image_on_load(
    mut commands: Commands,
    computed_viewport: Query<&ComputedViewport, Changed<ComputedViewport>>,
    mut uncentered_images: Query<(Entity, &Sprite, &mut Transform), With<ImageNotYetCentered>>,
) {
    if uncentered_images.iter().count() == 0 {
        return;
    }

    if computed_viewport.iter().count() == 0 {
        return;
    }

    let viewport = match computed_viewport.iter().next() {
        Some(data) => data,
        None => {
            error!("Canvas data not found");
            return;
        }
    };

    let (entity, sprite, mut transform) = match uncentered_images.iter_mut().next() {
        Some((entity, sprite, transform)) => (entity, sprite, transform),
        None => {
            return;
        }
    };

    info!("Centering image: {:?}", sprite.image.path());
    transform.translation = viewport.translation;

    commands.entity(entity).remove::<ImageNotYetCentered>();
}

pub fn scale_image_on_load(
    mut commands: Commands,
    computed_viewport: Query<&ComputedViewport>,
    window: Query<&Window>,
    mut uninitialized_images: Query<(Entity, &Sprite, &mut Transform), With<ImageNotYetScaled>>,
    mut main_camera: Query<
        &mut OrthographicProjection,
        (With<MainCamera>, Without<ImageNotYetScaled>),
    >,
    images: Res<Assets<Image>>,
) {
    if computed_viewport.iter().count() == 0 {
        return;
    }

    let viewport = match computed_viewport.iter().next() {
        Some(data) => data,
        None => {
            error!("Canvas data not found");
            return;
        }
    };

    let (entity, sprite, mut transform) = match uninitialized_images.iter_mut().next() {
        Some((entity, sprite, transform)) => (entity, sprite, transform),
        None => {
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

    transform.translation = viewport.translation;

    info!("Image path: {:?}", sprite.image.path());
    let image_size = match images.get(&sprite.image) {
        Some(image) => Vec2::new(image.width() as f32 * 2., image.height() as f32 * 2.),
        None => {
            error!("Image not found");
            return;
        }
    };

    // Adjust camera to fit image.
    let mut projection = match main_camera.iter_mut().next() {
        Some(projection) => projection,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // WILO: I've got the viewport data, how do I center
    // the image in the viewport now?

    info!("image size: {:#?}", image_size);
    info!("viewport: {:#?}", viewport);
    info!("projection.scale: {:#?}", projection.scale);

    // image_size.x = 2560
    // image_size.y = 1440
    // viewport.width = 1920
    // viewport.height = 1080
    //
    // viewport_size / image_size = scale_factor
    // 1920 / 2560 = 0.75
    let viewport_height_padding = viewport.height - MAGIC_NUMBER_UI;
    let viewport_width_padding = viewport.width - MAGIC_NUMBER_UI;
    let height_scale_factor = image_size.y / viewport_height_padding;
    let width_scale_factor = image_size.x / viewport_width_padding;

    info!("height scale factor: {}", height_scale_factor);
    info!("width scale factor: {}", width_scale_factor);

    projection.scale = if height_scale_factor > width_scale_factor {
        info!("height scale factor");
        height_scale_factor
    } else {
        info!("width scale factor");
        width_scale_factor
    };

    info!("projection: {:#?}", projection.scale);

    commands.entity(entity).remove::<ImageNotYetScaled>();
    commands.entity(entity).insert(ImageNotYetCentered);
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
    commands.spawn((
        Name::new("selected_image"),
        Sprite {
            image: next_image_handle.clone(),
            ..Default::default()
        },
        SelectedImage,
        ImageLoading(next_image_handle),
        ImageNotYetScaled,
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

    debug!("Current file name: {}", current_file_name);

    commands.spawn(CurrentFileNameLabelUpdateNeeded(current_file_name));
}

pub fn compute_canvas_viewport_data(
    mut commands: Commands,
    mut computed_data: Query<&mut ComputedViewport>,
    window: Query<&Window>,
    main_camera: Query<&OrthographicProjection, With<MainCamera>>,
    app_data: Res<AppData>,
) {
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

    let window_width = window.physical_width() as f32;
    let window_height = window.physical_height() as f32;

    let viewport_width_percentage = 1. - app_data.config.settings.ui_panel.size.width_percentage;
    let viewport_height_percentage = 1. - app_data.config.settings.ui_panel.size.height_percentage;

    let viewport_width = window_width * viewport_width_percentage;
    let viewport_height = window_height * viewport_height_percentage;

    let canvas_width = viewport_width - MAGIC_NUMBER_UI;
    let canvas_height = viewport_height - MAGIC_NUMBER_UI;

    let x_offset = window_width - canvas_width - MAGIC_NUMBER_UI;
    let y_offset = window_height - canvas_height - MAGIC_NUMBER_UI;

    let scaled_viewport_width = canvas_width * main_camera_projection.scale;
    let scaled_viewport_height = canvas_height * main_camera_projection.scale;
    let scaled_x_offset = x_offset / 4. * main_camera_projection.scale;
    let scaled_y_offset = y_offset / 4. * main_camera_projection.scale;

    if scaled_viewport_width <= 0.0
        || scaled_viewport_height <= 0.0
        || scaled_x_offset <= 0.0
        || scaled_y_offset <= 0.0
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

        if data.width == scaled_viewport_width
            && data.height == scaled_viewport_height
            && data.translation.x == scaled_x_offset
            && data.translation.y == scaled_y_offset
        {
            return;
        }
    }

    let data = ComputedViewport {
        width: scaled_viewport_width,
        height: scaled_viewport_height,
        translation: Vec3::new(scaled_x_offset, scaled_y_offset, 0.0),
    };

    // 2560x1440
    // 2560 * 0.75 = 1920
    // 1440 * 0.90 = 1296

    // 127x35
    info!("Computed canvas viewport data: {:#?}", data);

    if computed_data.iter().count() == 0 {
        commands.spawn(data);
    } else {
        for mut computed in computed_data.iter_mut() {
            info!("Updating computed canvas viewport data");
            *computed = data.clone();
        }
    }
}

pub fn debug_viewport(
    mut commands: Commands,
    canvas_marker: Query<Entity, With<CanvasMarker>>,
    canvas_data: Query<&ComputedViewport, Changed<ComputedViewport>>,
) {
    if canvas_data.iter().count() == 0 {
        return;
    }

    // Remove old debug canvas data
    for entity in canvas_marker.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let canvas_data = match canvas_data.iter().next() {
        Some(data) => {
            if let Some(marker) = canvas_marker.iter().next() {
                commands.entity(marker).despawn_recursive();
            };

            data
        }
        None => {
            error!("Canvas data not found");
            return;
        }
    };

    let transform = Transform::from_translation(canvas_data.translation);
    let size = Vec2::new(canvas_data.width, canvas_data.height) / 2.;

    let debug_canvas_border = (
        Name::new("debug_canvas"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                transform,
                hollow: true,
                thickness: 5.0,
                render_layers: Some(MAIN_LAYER),
                color: Color::from(INDIAN_RED),
                ..ShapeConfig::default_2d()
            },
            size,
        ),
        MAIN_LAYER,
    );

    commands.spawn(debug_canvas_border);

    let debug_canvas_center = (
        Name::new("debug_canvas_center"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                transform,
                render_layers: Some(MAIN_LAYER),
                color: Color::from(LIMEGREEN),
                ..ShapeConfig::default_2d()
            },
            Vec2::new(20.0, 20.0),
        ),
        MAIN_LAYER,
    );

    commands.spawn(debug_canvas_center);
}

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
