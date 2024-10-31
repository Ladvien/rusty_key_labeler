use std::{fs, io::Write, path::PathBuf, string};

use bevy::prelude::*;
mod settings;
use settings::Config;
use yolo_io::{ImageLabelPair, YoloDataQualityReport, YoloProject, YoloProjectConfig};

#[derive(Resource, Debug, Clone)]
pub struct YoloProjectResource(YoloProject);

#[derive(Resource, Debug, Clone)]
pub struct AppData {
    index: usize,
    current_pair: Option<ImageLabelPair>,
}

#[derive(Component)]
pub struct CurrentImage;

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data =
        std::fs::read_to_string("rusty_key_labeler/config.yaml").expect("Unable to read file");
    let config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");
    let project = YoloProject::new(&config.project_config);
    println!("Project: {:#?}", project);
    let report = YoloDataQualityReport::generate(project.clone());

    match report {
        Some(report) => {
            let mut file = fs::File::create("report.json").expect("Unable to create file");
            file.write_all(report.as_bytes())
                .expect("Unable to write data to file");
        }
        None => todo!(),
    }

    let project_resource = YoloProjectResource(project);

    let app_data = AppData {
        index: 0,
        current_pair: None,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .insert_resource(project_resource)
        .insert_resource(app_data)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (zoom_system, translate_image_system, load_next_image_system),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    project_resource: Res<YoloProjectResource>,
    mut app_data: ResMut<AppData>,
) {
    let valid_pairs = project_resource.0.get_valid_pairs();
    println!("Valid pairs: {:#?}", valid_pairs.len());

    let first_image = valid_pairs.first().unwrap();
    app_data.current_pair = Some(first_image.clone());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(
                first_image
                    .clone()
                    .image_path
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        },
        CurrentImage,
    ));

    commands.spawn(Camera2dBundle::default());
}

pub fn load_next_image_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    query: Query<Entity, With<CurrentImage>>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        println!("Loading next image");

        // Remove current image
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        load_next_image(
            &mut commands,
            &asset_server,
            &mut app_data,
            &project_resource,
        );

        println!("Current pair: {:#?}", app_data.current_pair);
    }
}

fn load_next_image(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    app_data: &mut ResMut<AppData>,
    project_resource: &Res<YoloProjectResource>,
) {
    let valid_pairs = project_resource.0.get_valid_pairs();
    let next_index = app_data.index + 1;
    if next_index < valid_pairs.len() {
        let next_image = valid_pairs[next_index].clone().image_path.unwrap();
        let next_image = next_image.as_path().to_string_lossy().into_owned();
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(next_image),
                ..Default::default()
            },
            CurrentImage,
        ));
        app_data.index = next_index;
        app_data.current_pair = Some(valid_pairs[next_index].clone());
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
    mut query: Query<&mut Transform, With<Sprite>>,
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
