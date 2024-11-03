use std::{fs, io::Write};

use bevy::{
    color::palettes::css::RED,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2d, Mesh2dHandle},
};
mod settings;
use settings::Config;
use yolo_io::{ImageLabelPair, YoloDataQualityReport, YoloFile, YoloProject};

#[derive(Resource, Debug, Clone)]
pub struct YoloProjectResource(YoloProject);

#[derive(Resource, Debug, Clone)]
pub struct AppData {
    index: usize,
    current_pair: Option<ImageLabelPair>,
}

#[derive(Component)]
pub struct CurrentImage;

/*
TODO: Resize image to fit window
*/

const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);
/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data =
        std::fs::read_to_string("rusty_key_labeler/config.yaml").expect("Unable to read file");
    let config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");
    let project = YoloProject::new(&config.project_config);

    // let report = YoloDataQualityReport::generate(project.clone().unwrap());

    // match report {
    //     Some(report) => {
    //         let mut file = fs::File::create("report.json").expect("Unable to create file");
    //         file.write_all(report.as_bytes())
    //             .expect("Unable to write data to file");
    //     }
    //     None => todo!(),
    // }

    let project_resource = YoloProjectResource(project.unwrap());

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

#[derive(Debug, Clone)]
pub struct BevyYoloFile(pub YoloFile);

// Impl conversion of YoloFile to MaterialMesh2dBundle
impl BevyYoloFile {
    fn to_2d_bundle(
        &self,
        // image: &mut ResMut<Assets<Mesh>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Vec<MaterialMesh2dBundle<ColorMaterial>> {
        let mut bundles = Vec::new();
        let yolo_file = self.0.clone();

        for entry in yolo_file.entries.iter() {
            let x_center = entry.x_center;
            let y_center = entry.y_center;
            let width = entry.width;
            let height = entry.height;

            let rectangle_handle = Mesh2dHandle(meshes.add(Rectangle::new(width, height)));

            let color_material = ColorMaterial::from_color(RED);

            let bundle = MaterialMesh2dBundle {
                mesh: rectangle_handle,
                material: materials.add(color_material),
                transform: Transform::from_xyz(x_center, y_center, 0.0),
                ..Default::default()
            };

            bundles.push(bundle);
        }

        bundles
    }
}

// WILO: Trying to load image, get size, and then draw bounding boxes
//       to scale on the image.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    project_resource: Res<YoloProjectResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let valid_pairs = project_resource.0.get_valid_pairs();

    let first_image = valid_pairs.first().unwrap();
    let image_handle: Handle<Image> = asset_server.load(first_image.image_path.clone().unwrap());

    let label_file = first_image.label_file.clone().unwrap();
    let bevy_yolo_file = BevyYoloFile(label_file);

    // Get the size of the image
    let image = images.get(&image_handle).unwrap();
    let image_size = Vec2::new(image.width() as f32, image.height() as f32);

    let shapes = bevy_yolo_file.to_2d_bundle(&mut meshes, &mut materials);

    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        // sprite: todo!(),
        // transform: todo!(),
        // global_transform: todo!(),
        texture: image_handle,
        // visibility: todo!(),
        // inherited_visibility: todo!(),
        // view_visibility: todo!(),
        ..Default::default()
    });
}

pub fn load_selected_image_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
) {
    let valid_pairs = project_resource.0.get_valid_pairs();
    let selected_pair = valid_pairs.get(app_data.index).unwrap();
    let selected_image = selected_pair.image_path.clone().unwrap();
    let selected_image = selected_image.as_path().to_string_lossy().into_owned();

    // Remove current image
    for entity in commands.query::<Entity>().with::<CurrentImage>().iter() {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(selected_image),
            ..Default::default()
        },
        CurrentImage,
    ));

    app_data.current_pair = Some(selected_pair.clone());
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
