use bevy::{asset::LoadState, prelude::*, window::WindowResized};
use bevy_lunex::prelude::MainUi;
use bevy_ui_views::{VStack, VStackContainerItem, VStackUpdatedItems};
use std::path::Path;

use crate::{
    bounding_boxes::{BoundingBoxMarker, BoundingBoxPainter},
    resources::{AppData, YoloProjectResource},
    settings::{MAIN_LAYER, UI_LAYER},
    ui::{UiDataChanged, UiPanel, UI},
    Config, ImageData, ImageToLoad, MainCamera, SelectedImage, TestFlag, UiCamera, UiData,
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

    let vstack_eid = commands
        .spawn((
            Name::new("VStack"),
            VStack {
                text: "ExtendedScrollView".to_string(),
                position: Vec2::new(0.0, 0.0),
                percent_width: 25.0,
                percent_height: 100.0,
                layer: UI_LAYER,
                ..Default::default()
            },
        ))
        .id();

    app_data.ui_eid = Some(vstack_eid);
}

pub fn on_resize_system(
    commands: Commands,
    resize_reader: EventReader<WindowResized>,
    window: Query<&Window>,
    mut ui: ResMut<UI>,
) {
    if resize_reader.is_empty() {
        return;
    }

    if window.iter().count() > 1 {
        panic!("More than one window found");
    }

    // let window = window.single();
    // ui.on_window_resize(commands, window);
}

// pub fn setup_ui(
//     mut commands: Commands,
//     mut ui: ResMut<UI>,
//     asset_server: Res<AssetServer>,
//     window: Query<&Window>,
//     query: Query<&UiData>,
// ) {
//     if window.iter().count() > 1 {
//         panic!("More than one window found");
//     }

//     commands.spawn((
//         // Add this marker component provided by Lunex.
//         MainUi,
//         // Our camera bundle with depth 1000.0 because UI starts at `0` and goes up with each layer.
//         Camera2dBundle {
//             camera: Camera {
//                 // Render the UI on top of everything else.
//                 order: 1,
//                 ..default()
//             },
//             transform: Transform::from_xyz(0.0, 0.0, 1000.0),
//             ..default()
//         },
//         UI_LAYER,
//         UiCamera,
//     ));

//     ui.paint_ui(commands, asset_server, window.single(), None);
// }

// pub fn update_ui_panel(
//     mut commands: Commands,
//     change_flag: Query<Entity, With<UiDataChanged>>,
//     main_ui: Query<Entity, With<MainUi>>,
//     mut ui: ResMut<UI>,
//     window: Query<&Window>,
//     asset_server: Res<AssetServer>,
//     query: Query<&ImageData, With<SelectedImage>>,
// ) {
//     if change_flag.iter().count() == 0 {
//         return;
//     }

//     if window.iter().count() > 1 {
//         panic!("More than one window found");
//     }

//     if change_flag.iter().count() > 1 {
//         panic!("More than one UI panel found");
//     }

//     // Remove the UiDataChanged component
//     for entity in change_flag.iter() {
//         commands.entity(entity).remove::<UiDataChanged>();
//     }

//     // Despawn the old UI panel
//     // for entity in main_ui.iter() {
//     //     commands.entity(entity).despawn_recursive();
//     // }

//     // Paint the UI panel
//     let data = query.iter().next();
//     ui.paint_ui(commands, asset_server, window.single(), data);
// }

pub fn on_image_loaded_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &ImageToLoad), With<ImageToLoad>>,
    app_data: Res<AppData>,
    mut has_run: Local<bool>,
) {
    // TODO: Clean up unwrap.

    const CLR_1: Color = Color::srgb(0.168, 0.168, 0.168);
    const CLR_2: Color = Color::srgb(0.109, 0.109, 0.109);
    const BORDER_COLOR: Color = Color::srgb(0.569, 0.592, 0.647);
    const CLR_4: Color = Color::srgb(0.902, 0.4, 0.004);

    if let Some(ui_eid) = app_data.ui_eid {
        if *has_run {
            return;
        }

        let mut items = Vec::new();
        for i in 0..10 {
            items.push(VStackContainerItem {
                text: format!("Item {}", i),
                background_color: if i % 2 == 0 { CLR_1 } else { CLR_2 },
                ..Default::default()
            });
        }
        commands.spawn(VStackUpdatedItems {
            vstack_eid: ui_eid,
            items,
        });

        println!("UI updated");
        *has_run = true;
    }

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

pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    query: Query<Entity, With<ImageToLoad>>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        app_data.index += 1;
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
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

    let next_image = valid_pairs[app_data.index as usize]
        .clone()
        .image_path
        .unwrap();
    let next_image = next_image.as_path().to_string_lossy().into_owned();

    let ui_data = UiData {
        stem: valid_pairs[app_data.index as usize].name.clone(),
        image_path: valid_pairs[app_data.index as usize]
            .image_path
            .clone()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
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

    // Remove ImageToLoad component
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn paint_bounding_boxes_system(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &ImageData), With<SelectedImage>>,
    old_bounding_boxes: Query<Entity, With<BoundingBoxMarker>>,
    project_resource: Res<YoloProjectResource>,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
) {
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

        if let Some((image_eid, image_data)) = query.iter().next() {
            let image = images.get(&image_data.image).unwrap();
            let image_size = Vec2::new(image.width() as f32, image.height() as f32);

            let bounding_boxes = bb_painter.get_boxes(&yolo_file, image_size);
            for bounding_box in bounding_boxes {
                let child_id = commands.spawn(bounding_box).id();
                children.push(child_id);
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
