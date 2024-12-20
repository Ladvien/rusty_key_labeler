use crate::{
    ComputedCanvasViewportData, DebounceTimer, ImageLoading, ImageReady, TopRightPanelUI,
    UIBottomPanel, UILeftPanel, UiBasePanel, UiCamera,
};
use bevy::{asset::LoadState, prelude::*};

pub fn debounce_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DebounceTimer), With<DebounceTimer>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        commands.entity(entity).remove::<DebounceTimer>();
    }
}

pub fn image_state_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images_being_loaded: Query<(Entity, &ImageLoading)>,
) {
    // Check if images are still being loaded.
    for (entity, image_loading) in images_being_loaded.iter() {
        match asset_server.get_load_state(&image_loading.0) {
            Some(image_handle) => match image_handle {
                LoadState::Loaded => {
                    info!("Image loaded");
                    commands.entity(entity).remove::<ImageLoading>();
                    commands
                        .entity(entity)
                        .insert(ImageReady(image_loading.0.clone()));
                }
                LoadState::NotLoaded => {
                    info!("Image not loaded");
                }
                LoadState::Loading => {
                    info!("Image loading");
                }
                LoadState::Failed(arc) => {
                    error!("Image failed to load: {:?}", arc);
                }
            },
            None => {
                error!("Image handle not found");
                return;
            }
        }
    }
}

pub fn compute_canvas_viewport_data(
    mut commands: Commands,
    top_right_hands_panel: Query<&ComputedNode, With<TopRightPanelUI>>,
    left_ui_panel_transform: Query<&ComputedNode, With<UILeftPanel>>,
    base_panel_transform: Query<&ComputedNode, With<UiBasePanel>>,
    bottom_ui_panel_transform: Query<&ComputedNode, With<UIBottomPanel>>,
    mut computed_data: Query<&mut ComputedCanvasViewportData>,
    window: Query<&Window>,
    main_camera: Query<&OrthographicProjection, With<UiCamera>>,
) {
    let top_right_cnode = match top_right_hands_panel.iter().next() {
        Some(cnode) => cnode,
        None => {
            error!("Top right panel not found");
            return;
        }
    };

    let left_ui_panel_cnode = match left_ui_panel_transform.iter().next() {
        Some(cnode) => cnode,
        None => {
            error!("Left UI panel transform not found");
            return;
        }
    };

    let base_panel_cnode = match base_panel_transform.iter().next() {
        Some(cnode) => cnode,
        None => {
            error!("Base panel transform not found");
            return;
        }
    };

    let bottom_ui_panel_cnode = match bottom_ui_panel_transform.iter().next() {
        Some(cnode) => cnode,
        None => {
            error!("Bottom panel transform not found");
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

    let projection = match main_camera.iter().next() {
        Some(projection) => projection,
        None => {
            error!("Main camera not found");
            return;
        }
    };

    // TODO: No idea why inverse_scale_factor is needed.
    // let x_canvas_offset =
    //     left_ui_panel_cnode.size().x / 2. * left_ui_panel_cnode.inverse_scale_factor();
    // let y_canvas_offset =
    //     bottom_ui_panel_cnode.size().y / 2. * bottom_ui_panel_cnode.inverse_scale_factor();

    // WILO: This calculates size correctly for MAIN_CAMERA.
    let canvas_width = (top_right_cnode.size().x) * projection.scale;
    let canvas_height = (top_right_cnode.size().y) * projection.scale;

    info!("projection: {:#?}", projection);
    info!("top_right_cnode: {:#?}", top_right_cnode.size());
    info!("Canvas width: {}, height: {}", canvas_width, canvas_height);
    info!("Window size: {:#?}", window.physical_size());

    if canvas_width <= 0.0 || canvas_height <= 0.0 {
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

        if data.width == canvas_width && data.height == canvas_height
        // && data.x_offset == x_canvas_offset
        // && data.y_offset == y_canvas_offset
        {
            return;
        }
    }

    let data = ComputedCanvasViewportData {
        // x_offset: x_canvas_offset,
        // y_offset: y_canvas_offset,
        x_offset: 0.0,
        y_offset: 0.0,
        width: canvas_width,
        height: canvas_height,
        // translation: Vec3::new(
        //     canvas_width / 2. - x_canvas_offset,
        //     canvas_height / 2. - y_canvas_offset,
        //     0.0,
        // ),
        translation: Vec3::new(0., 0., 0.),
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
