use bevy::{
    asset::LoadState,
    color::palettes::{css::*, tailwind::RED_950},
    core_pipeline::bloom::BloomSettings,
    prelude::*,
};
use bevy_vector_shapes::prelude::*;
use std::path::Path;

use crate::{
    resources::{AppData, YoloProjectResource},
    BoundingBox, Config, ImageData, ImageToLoad, SelectedImage,
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
    ));
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
) {
    // TODO: Kludge. Fix this.
    let num_bounding_boxes = old_bounding_boxes.iter().count();
    if num_bounding_boxes > 0 {
        return;
    }

    let mut children = Vec::new();

    if let Some((image_eid, image_data)) = query.iter().next() {
        let image = images.get(&image_data.image).unwrap();
        let image_size = Vec2::new(image.width() as f32, image.height() as f32);

        for (index, entry) in image_data.yolo_file.entries.iter().enumerate() {
            let scaled_x_center = entry.x_center * image_size.x;
            let scaled_y_center = entry.y_center * image_size.y;
            let scaled_width = entry.width * image_size.x;
            let scaled_height = entry.height * image_size.y;

            let bounding_box_transform = Transform::from_translation(Vec3::new(
                scaled_x_center - image_size.x / 2.,
                (scaled_y_center - image_size.y / 2.) * -1.,
                0.,
            ));

            let size = Vec2::new(scaled_width, scaled_height);

            let bounding_box_settings = config.settings.bounding_boxes.clone();

            let bounding_box_eid = commands
                .spawn((
                    Name::new(format!("bounding_box_{}", index)),
                    ShapeBundle::rect(
                        &ShapeConfig {
                            color: Color::srgba_u8(255, 66, 66, 255),
                            transform: bounding_box_transform,
                            hollow: true,
                            thickness: bounding_box_settings.thickness,
                            // corner_radii: Vec4::splat(0.3),
                            ..ShapeConfig::default_2d()
                        },
                        size,
                    ),
                    BoundingBox,
                ))
                .id();

            children.push(bounding_box_eid);
        }
        commands.entity(image_eid).push_children(&children);
    }
}

pub fn zoom_system(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
) {
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