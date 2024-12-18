use bevy::{asset::LoadState, prelude::*};
use bevy_ui_views::VStackUpdatedItems;
use yolo_io::ImageLabelPair;

use crate::{
    bounding_boxes::{BoundingBoxMarker, BoundingBoxPainter},
    resources::AppData,
    settings::MAIN_LAYER,
    ui::{
        CurrentFileNameLabelUpdateNeeded, UIBottomPanel, UILeftPanel, Ui, UiBasePanel,
        UiLabelingIndexUpdateNeeded,
    },
    Config, DebounceTimer, ImageLoading, ImageReady, ImageWithUninitializedScale, MainCamera,
    SelectedImage, TopRightPanelUI,
};

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    asset_server: Res<AssetServer>,
) {
    app_data.index = 0;
    let valid_pairs = app_data.yolo_project.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let image_handle = asset_server.load::<Image>(first_image_path.clone());

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2d::default(),
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        // BloomSettings {
        //     intensity: 0.1,
        //     high_pass_frequency: 0.1,
        //     ..Default::default()
        // },
        MAIN_LAYER,
        MainCamera,
    ));

    start_image_load(
        &mut commands,
        asset_server,
        app_data.index,
        valid_pairs.len() as isize - 1,
        0.0,
        valid_pairs,
    );
}

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

pub fn center_image_on_load(
    mut commands: Commands,
    canvas_data: Query<&ComputedCanvasViewportData>,
    mut uninitialized_images: Query<
        (Entity, &Sprite, &mut Transform),
        With<ImageWithUninitializedScale>,
    >,
    mut main_camera: Query<
        (&mut OrthographicProjection, &mut Transform),
        (With<MainCamera>, Without<ImageWithUninitializedScale>),
    >,
    images: Res<Assets<Image>>,
) {
    if canvas_data.iter().count() == 0 {
        return;
    }

    let canvas_data = match canvas_data.iter().next() {
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

    transform.translation = Vec3::new(canvas_data.x_offset, canvas_data.y_offset, 0.);

    info!("Image path: {:?}", sprite.image.path());
    let image_size = match images.get(&sprite.image) {
        Some(image) => Vec2::new(image.width() as f32, image.height() as f32),
        None => {
            error!("Image not found");
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

    camera_transform.translation = Vec3::new(canvas_data.x_offset, canvas_data.y_offset, 0.);

    info!("Image size: {:?}", image_size);
    info!("Projection area: {:?}", projection.area);
    // TODO: Handle scaling in when the image isn't large enough.

    // if image_size.y >= image_size.x {
    //     scale_factor = image_size.y / (projection.area.height() - canvas_data.y_offset / 2.);
    // } else {
    //     scale_factor = image_size.x / (projection.area.width() - canvas_data.x_offset / 2.);
    // }

    projection.scale = 1.0;
    let mut scale_factor = 1.0;
    let height_scale_factor = image_size.y / (projection.area.height() - canvas_data.y_offset / 2.);
    let width_scale_factor = image_size.x / (projection.area.width() - canvas_data.x_offset / 2.);

    if height_scale_factor > width_scale_factor {
        scale_factor = height_scale_factor;
    } else {
        scale_factor = width_scale_factor;
    }

    info!("Scale factor: {}", scale_factor);
    projection.scale = scale_factor;

    commands
        .entity(entity)
        .remove::<ImageWithUninitializedScale>();
}

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
        ZIndex(-10),
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

pub fn bounding_boxes_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<
        (Entity, &Sprite),
        (
            With<SelectedImage>,
            With<ImageReady>,
            Without<BoundingBoxMarker>,
        ),
    >,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
    ui: Res<Ui>,
) {
    if query.iter().count() == 0 {
        return;
    }

    info!("Painting bounding boxes");

    let pair = match app_data.yolo_project.pair_at_index(app_data.index) {
        Some(pair) => pair,
        None => {
            error!("Pair not found");
            return;
        }
    };

    let yolo_file = match pair.label_file {
        Some(file) => file,
        None => {
            error!("Label file not found");
            return;
        }
    };

    let mut children = Vec::new();
    let mut ui_items = Vec::new();

    let (selected_image_eid, sprite) = match query.iter().next() {
        Some((eid, sprite)) => (eid, sprite),
        None => {
            error!("Selected image not found");
            return;
        }
    };

    info!("Selected image: {:?}", sprite.image.id());

    match images.get_mut(&sprite.image) {
        Some(image) => {
            // TODO: Keep an eye on this.
            // TODO: What happens if this fails continually?
            commands
                .entity(selected_image_eid)
                .try_insert(BoundingBoxMarker);

            let image_size = Vec2::new(image.width() as f32, image.height() as f32);

            for (index, entry) in yolo_file.entries.iter().enumerate() {
                //
                info!("Adding bounding box: {}", index);
                let bounding_box = bb_painter.get_box(index, entry, image_size);
                let child_id = commands.spawn(bounding_box).id();
                children.push(child_id);

                let color = bb_painter.get_color(entry.class);

                // TODO: I should preload all the color swatches, giving them a path.
                let image = ui.create_image_from_color(color);
                let image_handle = images.add(image);

                let item = ui.create_bounding_box_entry(
                    &app_data.yolo_project.config.export.class_map[&entry.class],
                    image_handle,
                );

                ui_items.push(item);
            }

            // Add bounding box references to UI
            if let Some(left_panel_eid) = app_data.left_panel_eid {
                info!("Updating left panel");
                commands.spawn(VStackUpdatedItems {
                    items: ui_items.clone(),
                    vstack_eid: left_panel_eid,
                });
            }
        }
        None => {
            error!("Image not found");
            return;
        }
    };
    if !children.is_empty() {
        info!("Adding children to selected image");
        commands.entity(selected_image_eid).add_children(&children);
    }
}

pub fn compute_canvas_viewport_data(
    mut commands: Commands,
    top_right_hands_panel: Query<&ComputedNode, With<TopRightPanelUI>>,
    left_ui_panel_transform: Query<&ComputedNode, With<UILeftPanel>>,
    base_panel_transform: Query<&ComputedNode, With<UiBasePanel>>,
    bottom_ui_panel_transform: Query<&ComputedNode, With<UIBottomPanel>>,
    mut computed_data: Query<&mut ComputedCanvasViewportData>,
) {
    let cnode = match top_right_hands_panel.iter().next() {
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

    // TODO: No idea why inverse_scale_factor is needed.
    let x_canvas_offset =
        left_ui_panel_cnode.size().x / 2. * left_ui_panel_cnode.inverse_scale_factor();
    let y_canvas_offset =
        bottom_ui_panel_cnode.size().y / 2. * bottom_ui_panel_cnode.inverse_scale_factor();

    let width = cnode.size().x - x_canvas_offset;
    let height = cnode.size().y - y_canvas_offset;

    if width <= 0.0 || height <= 0.0 {
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

        if data.width == width
            && data.height == height
            && data.x_offset == x_canvas_offset
            && data.y_offset == y_canvas_offset
        {
            return;
        }
    }

    let canvas_width = (width + x_canvas_offset) / 2.;
    let canvas_height = (height + y_canvas_offset) / 2.;

    let data = ComputedCanvasViewportData {
        x_offset: x_canvas_offset,
        y_offset: y_canvas_offset,
        width: canvas_width,
        height: canvas_height,
    };

    if computed_data.iter().count() == 0 {
        commands.spawn(data);
        return;
    }

    for mut computed in computed_data.iter_mut() {
        *computed = data.clone();
    }
}

#[derive(Debug, Clone, Default, Component)]
pub struct CanvasMarker;

#[derive(Debug, Clone, Default, Component)]
pub struct ComputedCanvasViewportData {
    pub x_offset: f32,
    pub y_offset: f32,
    pub width: f32,
    pub height: f32,
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

    // let debug_canvas = (
    //     Name::new("debug_canvas"),
    //     CanvasMarker,
    //     ShapeBundle::rect(
    //         &ShapeConfig {
    //             transform: Transform::from_translation(Vec3::new(
    //                 canvas_data.x_offset,
    //                 canvas_data.y_offset,
    //                 99.0,
    //             )),
    //             // hollow: true,
    //             // thickness: 50.0,
    //             ..ShapeConfig::default_2d()
    //         },
    //         Vec2::new(canvas_data.width, canvas_data.height),
    //     ),
    //     MAIN_LAYER,
    // );

    // commands.spawn(debug_canvas);

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
