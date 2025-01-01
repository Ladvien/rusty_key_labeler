use bevy::color::palettes::css::{INDIAN_RED, LIMEGREEN};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view;
use bevy_inspector_egui::egui::viewport;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, ShapeBundle},
};

use crate::utils::create_image_from_color;
use crate::{settings::MAIN_LAYER, MainCamera};
use crate::{
    CanvasMarker, ComputedViewport, FocusViewport, TopRightPanelUI, UninitializedRenderTarget,
};

pub fn compute_viewport(
    mut commands: Commands,
    mut main_camera: Query<&mut Camera, With<MainCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut viewport: Query<
        (Entity, &ComputedNode, &mut ImageNode, &GlobalTransform),
        With<TopRightPanelUI>,
    >,
    uninitialized_render_target: Query<Entity, With<UninitializedRenderTarget>>,
) {
    if uninitialized_render_target.iter().count() == 0 {
        return;
    }

    let mut main_camera = match main_camera.iter_mut().next() {
        Some(camera) => camera,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    let (viewport_eid, viewport_computed_node, mut viewport_image_node, viewport_global_transform) =
        match viewport.iter_mut().next() {
            Some((
                viewport_eid,
                viewport_computed_node,
                viewport_image_node,
                viewport_global_transform,
            )) => (
                viewport_eid,
                viewport_computed_node,
                viewport_image_node,
                viewport_global_transform,
            ),
            None => {
                error!("Viewport not found");
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

    let computed_viewport_size = viewport_computed_node.size();

    if computed_viewport_size.x <= 0.0 || computed_viewport_size.y <= 0.0 {
        error!("Viewport size is invalid");
        return;
    };

    let canvas_image = create_image_from_color(
        Color::from(Srgba::new(0.0, 0.0, 0.0, 0.0)),
        computed_viewport_size.x as u32,
        computed_viewport_size.y as u32,
    );

    let canvas_image_handle = images.add(canvas_image);
    viewport_image_node.image = canvas_image_handle.clone();
    main_camera.target = RenderTarget::Image(viewport_image_node.image.clone());

    let computed_viewport = ComputedViewport {
        width: computed_viewport_size.x,
        height: computed_viewport_size.y,
        translation: viewport_global_transform.translation(),
    };

    commands.entity(viewport_eid).insert(computed_viewport);
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

pub fn fit_to_viewport(
    mut commands: Commands,
    target: Query<(Entity, &FocusViewport), (Added<FocusViewport>, Without<MainCamera>)>,
    mut main_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
    computed_viewport: Query<&ComputedViewport>,
) {
    if computed_viewport.iter().count() == 0 {
        return;
    }

    if target.iter().count() == 0 {
        return;
    }

    let mut projection = main_camera.single_mut();
    let viewport = computed_viewport.single();

    for (entity, target) in target.iter() {
        let width_scale_factor = target.width / viewport.width;
        let height_scale_factor = target.height / viewport.height;

        // Set the camera's projection to fit the larger dimension of the target.
        debug!(
            "Width scale factor: {}, Height scale factor: {}",
            width_scale_factor, height_scale_factor
        );

        projection.scale = if height_scale_factor > width_scale_factor {
            debug!("Selected {:#?}", height_scale_factor);
            height_scale_factor
        } else {
            debug!("Selected {:#?}", width_scale_factor);
            width_scale_factor
        };

        debug!("Removing FocusViewport");
        commands.entity(entity).remove::<FocusViewport>();
    }
}
