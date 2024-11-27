use bevy::{asset::LoadState, prelude::*};
use bevy_ui_views::{VStack, VStackContainerItem, VStackUpdatedItems};
use std::path::Path;

use crate::{
    bounding_boxes::{BoundingBoxMarker, BoundingBoxPainter},
    resources::{AppData, YoloProjectResource},
    settings::{MAIN_LAYER, UI_LAYER},
    ui::{create_image_from_color, UiDataChanged, UI},
    Config, DebounceTimer, ImageData, ImageToLoad, MainCamera, SelectedImage, UIBottomPanel,
    UILeftPanel, UiCamera, UiData,
};

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    asset_server: Res<AssetServer>,
) {
    app_data.index = 0;
    let valid_pairs = project_resource.0.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let image_handle = asset_server.load::<Image>(first_image_path.clone());

    commands.spawn((
        Name::new("selected_image"),
        SpriteBundle {
            texture: image_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..Default::default()
        },
        ImageToLoad {
            path: first_image_path,
            yolo_file: selected_pair.label_file.unwrap(),
        },
        MAIN_LAYER,
    ));

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2dBundle {
            // camera: Camera {
            //     hdr: true,
            //     ..default()
            // },
            transform: Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // BloomSettings {
        //     intensity: 0.1,
        //     high_pass_frequency: 0.1,
        //     ..Default::default()
        // },
        MAIN_LAYER,
        MainCamera,
    ));

    commands.spawn((
        Name::new("ui_camera"),
        Camera2dBundle {
            camera: Camera {
                // Render the UI on top of everything else.
                order: 1,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        UI_LAYER,
        UiCamera,
    ));

    // Spawn the UI Container
    let ui_eid = commands
        .spawn((
            Name::new("UIContainer"),
            NodeBundle {
                // Here is where all the styling goes for the container, duh.
                style: Style {
                    flex_direction: FlexDirection::Column,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                // background_color: BackgroundColor(Color::srgba(0.66, 0.03, 0.03, 0.03)), // DEBUG
                ..default()
            },
            UILeftPanel,
            UI_LAYER,
        ))
        .id();

    let vstack_eid = commands
        .spawn((
            Name::new("VStack"),
            VStack {
                text: "ExtendedScrollView".to_string(),
                position: Vec2::new(0.0, 0.0),
                percent_width: 25.0,
                percent_height: 90.0,
                layer: UI_LAYER,
                ..Default::default()
            },
        ))
        .id();

    app_data.ui_eid = Some(vstack_eid);

    commands.entity(ui_eid).push_children(&[vstack_eid]);

    let bottom_ui_eid = commands
        .spawn((
            Name::new("bottom_ui_panel"),
            NodeBundle {
                // Here is where all the styling goes for the container, duh.
                style: Style {
                    // left: Val::Px(0.0),
                    // top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    min_height: Val::Percent(10.0),
                    // height: Val::Percent(100.0),
                    // padding: UiRect::all(Val::Px(0.0)),
                    // margin: UiRect::all(Val::Px(0.0)),
                    // border: UiRect::all(Val::Px(0.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 1.0)),
                ..default()
            },
            UIBottomPanel,
            UI_LAYER,
        ))
        .id();

    commands.entity(ui_eid).push_children(&[bottom_ui_eid]);
}

pub fn on_image_loaded_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &ImageToLoad), With<ImageToLoad>>,
    app_data: Res<AppData>,
) {
    if let Some((entity, image_to_load)) = query.iter().next() {
        let image_handle: Handle<Image> = asset_server.load(image_to_load.path.clone());

        match asset_server.get_load_state(&image_handle) {
            Some(state) => {
                if state == LoadState::Loaded {
                    // Remove ImageToLoad component and add SelectedImage component
                    commands.entity(entity).remove::<ImageToLoad>();
                    commands.entity(entity).insert(SelectedImage);

                    let file_stem = Path::new(&image_to_load.path)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();

                    let image = images.get(&image_handle).unwrap();

                    let image_data = ImageData {
                        path: image_to_load.path.clone(),
                        stem: file_stem,
                        image: image_handle,
                        width: image.width() as f32,
                        height: image.height() as f32,
                        yolo_file: image_to_load.yolo_file.clone(),
                        index: app_data.index,
                        total_images: app_data.total_images,
                    };

                    // println!("Image loaded: {:#?}", image_data);
                    commands.entity(entity).insert((image_data, UiDataChanged));
                }
            }
            None => {
                println!("Image not loaded");
            }
        }
    }
}

#[warn(clippy::too_many_arguments)]
pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    config: Res<Config>,
    mut query: Query<Entity, With<ImageToLoad>>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
    time: Res<Time>,
    mut debounce_timer: Query<(Entity, &mut DebounceTimer)>,
) {
    if query.iter().count() > 0 {
        return;
    }

    for (entity, mut timer) in debounce_timer.iter_mut() {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        commands.entity(entity).despawn();
    }

    // TODO: Needs to be 'pressed', but need to debounce.
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        app_data.index += 1;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
        app_data.index -= 1;
    } else {
        return;
    }

    let valid_pairs = project_resource.0.get_valid_pairs();

    if app_data.index < 0 {
        app_data.index = valid_pairs.len() as isize - 1;
    }

    if app_data.index >= valid_pairs.len() as isize {
        app_data.index = 0;
    }

    // Despawn selected image
    for entity in query_selected_images.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // TODO: Clean up unwraps.
    let next_image = valid_pairs[app_data.index as usize]
        .clone()
        .image_path
        .unwrap();
    let next_image = next_image.as_path().to_string_lossy().into_owned();

    let ui_data = UiData {
        stem: valid_pairs[app_data.index as usize].name.clone(),
        // TODO: Clean up unwraps.
        image_path: valid_pairs[app_data.index as usize]
            .image_path
            .clone()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        // TODO: Clean up unwraps.
        label_path: valid_pairs[app_data.index as usize]
            .clone()
            .label_file
            .unwrap()
            .path,
    };

    commands.spawn((
        Name::new("selected_image"),
        SpriteBundle {
            texture: asset_server.load::<Image>(next_image.clone()),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..Default::default()
        },
        ImageToLoad {
            path: next_image,
            yolo_file: valid_pairs[app_data.index as usize]
                .label_file
                .clone()
                .unwrap(),
        },
        ui_data,
        MAIN_LAYER,
    ));

    commands.spawn((
        Name::new("debounce_timer"),
        DebounceTimer {
            timer: Timer::from_seconds(config.settings.delay_between_images, TimerMode::Once),
        },
    ));

    // Remove ImageToLoad component
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    keyboard_input.clear();
}

pub fn paint_bounding_boxes_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &ImageData), With<SelectedImage>>,
    old_bounding_boxes: Query<Entity, With<BoundingBoxMarker>>,
    project_resource: Res<YoloProjectResource>,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
) {
    // TODO: Clean up unwraps.
    if old_bounding_boxes.iter().count() > 0 {
        return;
    }

    if let Some(yolo_file) = project_resource
        .0
        .pair_at_index(app_data.index)
        .unwrap()
        .label_file
    {
        let mut children = Vec::new();

        let mut ui_items = Vec::new();

        if let Some((image_eid, image_data)) = query.iter().next() {
            let image = images.get(&image_data.image).unwrap();
            let image_size = Vec2::new(image.width() as f32, image.height() as f32);

            for (index, entry) in yolo_file.entries.iter().enumerate() {
                let bounding_box = bb_painter.get_box(index, entry, image_size);
                let child_id = commands.spawn(bounding_box).id();
                children.push(child_id);

                let color = bb_painter.get_color(entry.class);
                let item = VStackContainerItem {
                    text: project_resource.0.config.export.class_map[&entry.class].clone(),
                    image: Some(create_image_from_color(&mut images, color)),
                    ..Default::default()
                };

                ui_items.push(item);
            }

            if let Some(ui_eid) = app_data.ui_eid {
                commands.spawn(VStackUpdatedItems {
                    items: ui_items,
                    vstack_eid: ui_eid,
                });
            }

            if children.is_empty() {
                return;
            }
            commands.entity(image_eid).push_children(&children);
        }
    }
}

pub fn zoom_system(
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
) {
    // TODO: I need to figure out how to separate the UI from the camera zoom.
    //       maybe layers?
    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();

        if keyboard_input.pressed(config.settings.key_map.zoom_in) {
            log_scale -= config.settings.zoom_factor * time.delta_seconds();
        }
        if keyboard_input.pressed(config.settings.key_map.zoom_out) {
            log_scale += config.settings.zoom_factor * time.delta_seconds();
        }
        projection.scale = log_scale.exp();
    }
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
            translation.y += config.settings.pan_factor.y * time.delta_seconds();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_down) {
            translation.y -= config.settings.pan_factor.y * time.delta_seconds();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_left) {
            translation.x -= config.settings.pan_factor.x * time.delta_seconds();
        }
        if keyboard_input.pressed(config.settings.key_map.pan_right) {
            translation.x += config.settings.pan_factor.x * time.delta_seconds();
        }
        transform.translation = translation;
    }
}
