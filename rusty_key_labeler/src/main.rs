use bevy::prelude::*;
mod settings;
use settings::Config;
use yolo_io::{YoloProject, YoloProjectConfig};

#[derive(Resource)]
pub struct YoloProjectResource(YoloProject);

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data =
        std::fs::read_to_string("rusty_key_labeler/config.yaml").expect("Unable to read file");
    let config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");

    // println!("{:#?}", config);

    let mut project = YoloProject::new(&config.project_config);
    let results = project.validate();

    let project_resource = YoloProjectResource(project);

    println!("{:#?}", results);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .insert_resource(project_resource)
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_system, translate_image_system))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    project_resource: Res<YoloProjectResource>,
) {
    // println!("{:?}", first_image);

    // let image_path = first_image.unwrap().1;

    commands.spawn(Camera2dBundle::default());
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load(image_path),
    //     ..default()
    // });
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
