use bevy::{asset::LoadState, prelude::*, window::WindowResized};
use bevy_vector_shapes::prelude::*;
use std::path::Path;

use crate::{
    resources::{AppData, YoloProjectResource},
    settings::{MAIN_LAYER, UI_LAYER},
    ui::{UiDataChanged, UiPanel, UI},
    utils::{get_bounding_box_transform, scale_dimensions, srgba_string_to_color},
    BoundingBox, Config, ImageData, ImageToLoad, MainCamera, SelectedImage, UiCamera,
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

    let window = window.single();
    ui.on_window_resize(commands, window);
}

pub fn setup_ui(mut commands: Commands, ui: Res<UI>) {
    commands.spawn(ui.get_ui_bundle());
}

pub fn update_ui_panel(
    mut commands: Commands,
    mut change_flag: Query<Entity, With<UiDataChanged>>,
    mut old_ui: Query<Entity, With<UiPanel>>,
    ui: Res<UI>,
    window: Query<&Window>,
) {
    if change_flag.iter().count() == 0 {
        return;
    }

    if window.iter().count() > 1 {
        panic!("More than one window found");
    }

    if change_flag.iter().count() > 1 {
        panic!("More than one UI panel found");
    }

    // Despawn the change flag
    for ui_panel in change_flag.iter_mut() {
        commands.entity(ui_panel).despawn_recursive();
    }

    // Despawn the UI panel
    for ui_panel in old_ui.iter_mut() {
        commands.entity(ui_panel).despawn_recursive();
    }

    commands.spawn(ui.get_ui_bundle());
}

pub fn on_image_loaded_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &ImageToLoad), With<ImageToLoad>>,
) {
    // TODO: Clean up unwrap.
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
                    };

                    commands.entity(entity).insert(image_data);
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
    let mut index = app_data.index;
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        index = app_data.index + 1;
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        index = app_data.index - 1;
    } else {
        return;
    }

    if index < 0 {
        index = project_resource.0.get_valid_pairs().len() as isize - 1;
    }

    println!("Loading next image");

    // Despawn selected image
    for entity in query_selected_images.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let valid_pairs = project_resource.0.get_valid_pairs();

    if index < valid_pairs.len() as isize {
        let next_image = valid_pairs[index as usize].clone().image_path.unwrap();
        let next_image = next_image.as_path().to_string_lossy().into_owned();
        commands.spawn((
            Name::new("selected_image"),
            SpriteBundle {
                texture: asset_server.load::<Image>(next_image.clone()),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..Default::default()
            },
            ImageToLoad {
                path: next_image,
                yolo_file: valid_pairs[index as usize].label_file.clone().unwrap(),
            },
            MAIN_LAYER,
        ));
        app_data.index = index;
    }

    // Remove ImageToLoad component
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn paint_bounding_boxes_system(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &ImageData), With<SelectedImage>>,
    old_bounding_boxes: Query<Entity, With<BoundingBox>>,
    config: Res<Config>,
    project_resource: Res<YoloProjectResource>,
) {
    if old_bounding_boxes.iter().count() > 0 {
        return;
    }

    let class_color_map = config.settings.bounding_boxes.class_color_map.clone();
    let mut children = Vec::new();

    if let Some((image_eid, image_data)) = query.iter().next() {
        let image = images.get(&image_data.image).unwrap();
        let image_size = Vec2::new(image.width() as f32, image.height() as f32);
        let bounding_box_settings = config.settings.bounding_boxes.clone();

        for (index, entry) in image_data.yolo_file.entries.iter().enumerate() {
            let (scaled_x_center, scaled_y_center, scaled_width, scaled_height) = scale_dimensions(
                entry.x_center,
                entry.y_center,
                entry.width,
                entry.height,
                image_size,
            );

            let bounding_box_transform =
                get_bounding_box_transform(scaled_x_center, scaled_y_center, image_size);

            let size = Vec2::new(scaled_width, scaled_height);

            let class_names_map = project_resource.0.config.export.class_map.clone();
            let class_name = class_names_map[&entry.class].clone();
            let class_color_string = class_color_map[&class_name].clone();

            if let Some(class_color) = srgba_string_to_color(&class_color_string) {
                let bounding_box_eid = commands
                    .spawn((
                        Name::new(format!("bounding_box_{}", index)),
                        ShapeBundle::rect(
                            &ShapeConfig {
                                color: class_color,
                                transform: bounding_box_transform,
                                hollow: true,
                                thickness: bounding_box_settings.thickness,
                                corner_radii: Vec4::splat(bounding_box_settings.corner_radius),
                                ..ShapeConfig::default_2d()
                            },
                            size,
                        ),
                        BoundingBox,
                        MAIN_LAYER,
                    ))
                    .id();

                children.push(bounding_box_eid);
            }
        }
        commands.entity(image_eid).push_children(&children);
    }
}

pub fn zoom_system(
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
    mut ui: ResMut<UI>,
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
