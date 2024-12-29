use crate::{resources::AppData, MainCamera};
use crate::{CanvasMarker, ComputedViewport, FocusViewport, ImageViewport, ViewportCamera};
use bevy::color::palettes::css::{INDIAN_RED, LIMEGREEN};
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, Viewport};
use bevy::render::view::RenderLayers;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, ShapeBundle},
};
pub const MAGIC_NUMBER_UI: f32 = 10.0;
pub const VIEWPORT_LAYER: RenderLayers = RenderLayers::layer(1);

pub fn setup_viewport(
    mut commands: Commands,
    // mut app_data: ResMut<AppData>,
    viewport: Query<(&ComputedNode), With<ImageViewport>>,
    mut viewport_camera: Query<&mut Camera, With<ViewportCamera>>,
) {
    if viewport.iter().count() == 0 {
        return;
    }

    let mut camera = match viewport_camera.iter_mut().next() {
        Some(camera) => camera,
        None => {
            error!("Viewport camera not found");
            return;
        }
    };

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(1920, 1080),
        depth: 0.0..1.0,
    });
}

pub fn compute_canvas_viewport(
    mut commands: Commands,
    mut computed_data: Query<&mut ComputedViewport>,
    window: Query<&Window>,
    mut main_camera: Query<
        (&mut Camera, &mut Transform, &OrthographicProjection),
        With<MainCamera>,
    >,
    app_data: Res<AppData>,
) {
    let window = match window.iter().next() {
        Some(window) => window,
        None => {
            error!("Window not found");
            return;
        }
    };

    let (mut camera, mut main_camera_transform, main_camera_projection) =
        match main_camera.iter_mut().next() {
            Some((camera, main_camera_transform, main_camera_projection)) => {
                (camera, main_camera_transform, main_camera_projection)
            }
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

    let x_offset = window_width - viewport_width - MAGIC_NUMBER_UI;
    let y_offset = window_height - viewport_height - MAGIC_NUMBER_UI;

    // info!("x_offset: {}", x_offset);
    // info!("y_offset: {}", y_offset);

    let scaled_viewport_width = viewport_width * main_camera_projection.scale;
    let scaled_viewport_height = viewport_height * main_camera_projection.scale;
    let scaled_x_offset = x_offset / 4. * main_camera_projection.scale;
    let scaled_y_offset = y_offset / 4. * main_camera_projection.scale;

    // let scaled_x_offset = -100.0 * main_camera_projection.scale;
    // let scaled_y_offset = 10.0; // * main_camera_projection.scale;

    info!("scaled_x_offset: {}", scaled_x_offset);
    info!("scaled_y_offset: {}", scaled_y_offset);

    // if scaled_viewport_width <= 0.0
    //     || scaled_viewport_height <= 0.0
    //     || scaled_x_offset <= 0.0
    //     || scaled_y_offset <= 0.0
    // {
    //     info!("Computed canvas viewport data is unavailable");
    //     return;
    // }

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
            debug!("Updating computed canvas viewport data");
            *computed = data.clone();
        }
    }
}

pub fn fit_to_viewport(
    mut commands: Commands,
    computed_viewport: Query<&ComputedViewport>,
    mut target: Query<(Entity, &FocusViewport), Without<MainCamera>>,
    mut main_camera: Query<(&mut OrthographicProjection, &mut Transform), With<MainCamera>>,
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

    let (entity, target_size) = match target.iter_mut().next() {
        Some((entity, not_yet_scaled)) => (entity, not_yet_scaled),
        None => {
            return;
        }
    };

    // Adjust camera to fit image.
    let (mut projection, mut camera_transform) = match main_camera.iter_mut().next() {
        Some((projection, camera_transform)) => (projection, camera_transform),
        None => {
            error!("Main camera not found");
            return;
        }
    };

    let (target_height, target_width) = (
        target_size.height * 2. * projection.scale,
        target_size.width * 2. * projection.scale,
    );

    debug!("target_height: {}", target_height);
    debug!("target_width: {}", target_width);
    debug!("viewport: {:#?}", viewport);
    debug!("projection.scale: {:#?}", projection.scale);

    // image_size.x = 2560
    // image_size.y = 1440
    // viewport.width = 1920
    // viewport.height = 1080
    //
    // viewport_size / image_size = scale_factor
    // 1920 / 2560 = 0.75
    let viewport_height_padding = viewport.height - MAGIC_NUMBER_UI;
    let viewport_width_padding = viewport.width - MAGIC_NUMBER_UI;
    let height_scale_factor = target_height / viewport_height_padding;
    let width_scale_factor = target_width / viewport_width_padding;

    debug!("height scale factor: {}", height_scale_factor);
    debug!("width scale factor: {}", width_scale_factor);

    // Set the camera's projection to fit the larger dimension of the target.
    projection.scale = if height_scale_factor > width_scale_factor {
        debug!("Selected height scale factor");
        height_scale_factor
    } else {
        debug!("Selected width scale factor");
        width_scale_factor
    };

    // 2560x1440
    // 2560 * 0.80 = 1920
    // 1440 * 0.90 = 1296

    // transform.translation = viewport.translation * projection.scale;
    let new_camera_translation = Vec3::new(
        viewport.translation.x - 512.0 + MAGIC_NUMBER_UI,
        viewport.translation.y - 144.0 + MAGIC_NUMBER_UI,
        0.0,
    );
    camera_transform.translation = new_camera_translation;
    debug!("projection: {:#?}", projection.scale);

    commands.entity(entity).remove::<FocusViewport>();
}

pub fn debug_viewport(
    mut commands: Commands,
    canvas_marker: Query<Entity, With<CanvasMarker>>,
    viewport: Query<&ComputedViewport, Changed<ComputedViewport>>,
) {
    if viewport.iter().count() == 0 {
        return;
    }

    // Remove old debug canvas data
    for entity in canvas_marker.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let viewport = match viewport.iter().next() {
        Some(data) => data,
        None => {
            error!("Canvas data not found");
            return;
        }
    };

    let transform = Transform::from_translation(viewport.translation);
    let size = Vec2::new(viewport.width, viewport.height) / 2.;

    let debug_canvas_border = (
        Name::new("debug_canvas"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                transform,
                hollow: true,
                thickness: 5.0,
                color: Color::from(INDIAN_RED),
                ..ShapeConfig::default_2d()
            },
            size,
        ),
    );

    commands.spawn(debug_canvas_border);

    let debug_canvas_center = (
        Name::new("debug_canvas_center"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                transform,
                color: Color::from(LIMEGREEN),
                ..ShapeConfig::default_2d()
            },
            Vec2::new(20.0, 20.0),
        ),
    );

    commands.spawn(debug_canvas_center);
}
