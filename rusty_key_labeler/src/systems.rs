use bevy::{asset::LoadState, prelude::*};
use bevy_ui_views::VStackUpdatedItems;
use std::path::Path;

use crate::{
    bounding_boxes::{BoundingBoxMarker, BoundingBoxPainter},
    resources::{AppData, YoloProjectResource},
    settings::{MAIN_LAYER, UI_LAYER},
    ui::{
        CurrentFileNameLabelUpdateNeeded, UIBottomPanel, UILeftPanel, Ui, UiLabelDataChanged,
        UiLabelingIndexUpdateNeeded,
    },
    Config, DebounceTimer, ImageData, ImageToLoad, MainCamera, SelectedImage,
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
}

#[warn(clippy::too_many_arguments)]
pub fn next_and_previous_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    config: Res<Config>,
    query_image_to_load: Query<Entity, With<ImageToLoad>>,
    query_selected_images: Query<Entity, With<SelectedImage>>,
    time: Res<Time>,
    mut debounce_timer: Query<(Entity, &mut DebounceTimer)>,
) {
    // This query ensures the image is loaded before we can move to the next one.
    if query_image_to_load.iter().count() > 0 {
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
        MAIN_LAYER,
    ));

    // Update index label
    let index_label = format!("{}/{}", app_data.index + 1, app_data.total_images + 1);
    commands.spawn(UiLabelingIndexUpdateNeeded(index_label));

    // Debounce timer
    commands.spawn((
        Name::new("debounce_timer"),
        DebounceTimer {
            timer: Timer::from_seconds(config.settings.delay_between_images, TimerMode::Once),
        },
    ));

    // Remove ImageToLoad component
    for entity in query_image_to_load.iter() {
        commands.entity(entity).despawn();
    }

    keyboard_input.clear();
}

pub fn load_image_system(
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
                        stem: file_stem.clone(),
                        image: image_handle,
                        width: image.width() as f32,
                        height: image.height() as f32,
                        yolo_file: image_to_load.yolo_file.clone(),
                        index: app_data.index,
                        total_images: app_data.total_images,
                    };

                    commands.spawn(CurrentFileNameLabelUpdateNeeded(file_stem));

                    // TODO: If we pass in the index and total_images, and number of
                    //      non-empty label files, we can use the same system
                    //      for progress bar.
                    commands.spawn(UiLabelingIndexUpdateNeeded(format!(
                        "{}/{}",
                        app_data.index + 1,
                        app_data.total_images + 1
                    )));

                    commands
                        .entity(entity)
                        .insert((image_data, UiLabelDataChanged));
                }
            }
            None => {
                println!("Image not loaded");
            }
        }
    }
}

pub fn paint_bounding_boxes_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<(Entity, &ImageData), With<SelectedImage>>,
    old_bounding_boxes: Query<Entity, With<BoundingBoxMarker>>,
    project_resource: Res<YoloProjectResource>,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
    ui: Res<Ui>,
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
                let image = ui.create_image_from_color(color);
                let image_handle = images.add(image);

                let item = ui.create_bounding_box_entry(
                    &project_resource.0.config.export.class_map[&entry.class],
                    image_handle,
                );

                ui_items.push(item);
            }

            if let Some(ui_eid) = app_data.ui_eid {
                commands.spawn(VStackUpdatedItems {
                    items: ui_items,
                    vstack_eid: ui_eid,
                });
            }

            if !children.is_empty() {
                commands.entity(image_eid).push_children(&children);
            }
        }
    }
}

// WILO: I'd like to stop changing the position of the camera
// and instead change the position and or scale of the image.
pub fn image_view_system(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<Config>,
    window: Query<&Window>,
    just_selected: Query<(Entity, &ImageData), Added<SelectedImage>>,
    mut transforms: ParamSet<(
        Query<(Entity, &ImageData, &mut Transform), Added<SelectedImage>>,
        Query<&Transform, With<UILeftPanel>>,
        Query<&Transform, With<UIBottomPanel>>,
    )>,
) {
    // 1. Ensure the image is maxed height or width according to the viewport size.
    // 2. Center the image on first selected.
    // 3. Allow panning and zooming.

    // WILO: Re-think approach, this isn't working.

    let window = window.iter().next().unwrap(); // TODO: handle

    for (entity, image_data) in just_selected.iter() {
        let mut left_panel_width: f32 = 0.0;
        let mut bottom_panel_height: f32 = 0.0;

        let left_panel_query = transforms.p1();
        match left_panel_query.get_single() {
            Ok(value) => left_panel_width = value.translation.x,
            Err(_) => {
                error!("Left panel not found");
                return;
            }
        };

        let bottom_panel_query = transforms.p2();
        match bottom_panel_query.get_single() {
            Ok(value) => bottom_panel_height = value.translation.y,
            Err(_) => {
                error!("Bottom panel not found");
                return;
            }
        };

        println!("Window width: {}", window.width());
        println!("Window height: {}", window.height());
        println!("Left panel width: {}", left_panel_width);
        println!("Bottom panel height: {}", bottom_panel_height);

        commands.spawn((
            Name::new("viewport"),
            NodeBundle {
                style: Style {
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    border: UiRect::all(Val::Px(4.0)),
                    width: Val::Px(window.width()),
                    height: Val::Px(window.height() - bottom_panel_height / 2.),
                    ..Default::default()
                },
                border_color: BorderColor::from(Color::srgba(0.1, 0.1, 1.0, 1.0)),
                background_color: BackgroundColor::from(Color::srgba(1.0, 0.1, 0.0, 1.0)),
                ..Default::default()
            },
            UI_LAYER,
        ));
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
