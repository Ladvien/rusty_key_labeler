use crate::utils::create_image_from_color;
use crate::{resources::AppData, settings::MAIN_LAYER, MainCamera};
use crate::{
    CanvasMarker, ComputedViewport, FocusViewport, ImageLoading, ImageReady, SelectedImage,
    TopRightPanelUI, UiCamera, UninitializedRenderTarget,
};
use bevy::color::palettes::css::{INDIAN_RED, LIMEGREEN};
use bevy::prelude::*;
use bevy::render::camera::{self, RenderTarget, SubCameraView, Viewport};
use bevy::render::view;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, ShapeBundle},
};
pub const MAGIC_NUMBER_UI: f32 = 10.0;

pub fn test(
    mut commands: Commands,
    window: Query<&Window>,
    mut main_camera: Query<(&mut Camera, &GlobalTransform), With<MainCamera>>,
    // mut ui_camera: Query<(&mut Camera, &GlobalTransform), (With<UiCamera>, Without<MainCamera>)>,
    mut viewport: Query<
        (Entity, &GlobalTransform, &ComputedNode, &mut ImageNode),
        With<TopRightPanelUI>,
    >,
    mut images: ResMut<Assets<Image>>,
    selected_image: Query<&ImageReady, With<ImageReady>>,
    uninitialized_render_target: Query<Entity, With<UninitializedRenderTarget>>,
) {
    if uninitialized_render_target.iter().count() == 0 {
        return;
    }

    let (mut main_camera, main_camera_transform) = match main_camera.iter_mut().next() {
        Some((camera, main_camera_transform)) => (camera, main_camera_transform),
        None => {
            error!("Main camera not found");
            return;
        }
    };

    let (viewport_eid, viewport_transform, viewport_computed_node, mut viewport_image_node) =
        match viewport.iter_mut().next() {
            Some((
                viewport_eid,
                viewport_transform,
                viewport_computed_node,
                viewport_image_node,
            )) => (
                viewport_eid,
                viewport_transform,
                viewport_computed_node,
                viewport_image_node,
            ),
            None => {
                error!("Viewport not found");
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

    let image_ready = match selected_image.iter().next() {
        Some(image_ready) => image_ready,
        None => {
            error!("Selected image not found");
            return;
        }
    };

    let uninitialized_render_target_eid = match uninitialized_render_target.iter().next() {
        Some(eid) => eid,
        None => {
            error!("UninitializedRenderTarget not found");
            return;
        }
    };

    let full_size = window.physical_size();
    let main_camera_position = main_camera_transform.translation().truncate();
    let viewport_position = viewport_transform.translation().truncate();

    info!("main_camera_position: {:#?}", main_camera_position);
    info!("viewport_position: {:#?}", viewport_position);

    // let x_offset = viewport_position.x;
    // let y_offset = viewport_position.y;

    // let offset = Vec2::new(x_offset, -y_offset);

    let size = viewport_computed_node.size();
    // let size = size.as_uvec2();

    // info!("offset: {:#?}", offset);

    // main_camera.sub_camera_view = Some(SubCameraView {
    //     full_size,
    //     offset,
    //     size,
    // });

    let physical_position = UVec2::new(viewport_position.x as u32, viewport_position.y as u32);
    let physical_size = UVec2::new(size.x as u32, size.y as u32);

    info!("physical_position: {:#?}", physical_position);
    info!("physical_size: {:#?}", physical_size);

    // let viewport = Some(Viewport {
    //     physical_position,
    //     physical_size,
    //     ..Default::default()
    // });

    if size.x <= 0.0 || size.y <= 0.0 {
        error!("Viewport size is invalid");
        return;
    };

    let canvas_image = create_image_from_color(
        Color::from(Srgba::new(0.0, 0.0, 0.0, 0.0)),
        size.x as u32 + 200,
        size.y as u32,
    );

    let canvas_image_handle = images.add(canvas_image);
    viewport_image_node.image = canvas_image_handle.clone();
    main_camera.target = RenderTarget::Image(viewport_image_node.image.clone());

    info!("Removing UninitializedRenderTarget");
    commands.entity(uninitialized_render_target_eid).despawn();
}

pub fn debug_viewport(
    mut commands: Commands,
    canvas_marker: Query<Entity, With<CanvasMarker>>,
    // viewport: Query<&ComputedViewport, Changed<ComputedViewport>>,
    main_camera: Query<(&Camera, &OrthographicProjection), With<MainCamera>>,
) {
    // if viewport.iter().count() == 0 {
    //     return;
    // }

    // Remove old debug canvas data
    for entity in canvas_marker.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // let viewport = match viewport.iter().next() {
    //     Some(data) => data,
    //     None => {
    //         error!("Canvas data not found");
    //         return;
    //     }
    // };

    let (camera, camera_projection) = match main_camera.iter().next() {
        Some((camera, camera_projection)) => (camera, camera_projection),
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // let transform = Transform::from_translation(viewport.translation);
    // let size = Vec2::new(viewport.width, viewport.height) / 2.;
    let size = match camera.viewport.clone() {
        Some(viewport) => Vec2::new(
            viewport.physical_size.x as f32,
            viewport.physical_size.y as f32,
        ),
        None => {
            error!("Viewport not found");
            return;
        }
    };

    let debug_canvas_border = (
        Name::new("debug_canvas"),
        CanvasMarker,
        ShapeBundle::rect(
            &ShapeConfig {
                // transform,
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
                // transform,
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
