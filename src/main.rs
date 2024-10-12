use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub zoom_factor: f32,
    // pub pan: Vec2,
}

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct Config {
    pub image_path: String,
    pub annotation_path: String,
    pub output_path: String,
    pub settings: Settings,
}

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data = std::fs::read_to_string("config.yaml").expect("Unable to read file");
    let config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");

    println!("{:#?}", config);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_system))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root_path = String::from("/Users/ladvien/Dropbox/graphics/ml_images/spriter_sheet_ider/combined_annotations/train/images/");
    let image_path = String::from("zx_spectrum_-_pac-mania_-_pac-man.png");
    let image_path = format!("{}{}", root_path, image_path);

    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load(image_path),
        ..default()
    });
}

pub fn zoom_system(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
) {
    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();
        if keyboard_input.pressed(KeyCode::KeyE) {
            log_scale -= config.settings.zoom_factor * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            log_scale += config.settings.zoom_factor * time.delta_seconds();
        }
        projection.scale = log_scale.exp();
    }
}
