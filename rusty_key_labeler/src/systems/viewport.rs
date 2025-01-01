use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

use crate::utils::create_image_from_color;
use crate::MainCamera;
use crate::{ComputedViewport, FocusViewport, TopRightPanelUI, UninitializedRenderTarget};

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

pub fn fit_to_viewport(
    mut commands: Commands,
    mut main_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
    target: Query<(Entity, &FocusViewport), (Added<FocusViewport>, Without<MainCamera>)>,
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
