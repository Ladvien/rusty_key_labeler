mod settings;

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
    window::WindowResized,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use std::fs;

use settings::Config;
use yolo_io::{ImageLabelPair, YoloFile, YoloProject};

#[derive(Resource, Debug, Clone)]
pub struct YoloProjectResource(YoloProject);

#[derive(Resource, Debug, Clone)]
pub struct AppData {
    index: usize,
    current_pair: Option<ImageLabelPair>,
}

// const CANVAS_LAYER: RenderLayers = RenderLayers::layer(0);
// const IMAGE_LAYER: RenderLayers = RenderLayers::layer(1);
// const BOUNDING_BOX_LAYER: RenderLayers = RenderLayers::layer(2);
const RESOLUTION_WIDTH: f32 = 1920.0;
const RESOLUTION_HEIGHT: f32 = 1080.0;

/*
TODO: Resize image to fit window
*/

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
        .init_resource::<Assets<ColorMaterial>>()
        .add_plugins((
            // DefaultPlugins,
            ShapePlugin,
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::new(),
        ))
        .insert_resource(config)
        .insert_resource(project_resource)
        .insert_resource(app_data)
        // Add color resources
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                zoom_system,
                translate_image_system,
                load_next_image_system,
                // load_labels_for_image_system,
                // fit_canvas_system,
            ),
        )
        .run();
}

// #[derive(Debug, Clone, Component)]
// pub struct BevyYoloFile(pub YoloFile);

// Impl conversion of YoloFile to MaterialMesh2dBundle

#[derive(Debug, Clone, Component)]
pub struct Canvas;

#[derive(Component)]
pub struct ImageToLoad;

#[derive(Debug, Clone, Component)]
pub struct BevyYoloFile(pub YoloFile);

// #[derive(Debug, Clone, Component)]
// pub struct YoloLabelBundle {
//     spatial_bundle: SpatialBundle,
//     rectangle: shapes::Rectangle,
// }

#[derive(Debug, Clone, Component)]
pub struct CanvasCamera;

// #[derive(Debug, Clone, Component)]
// pub struct ImageCamera;

// impl BevyYoloFile {
//     fn to_rectangle(&self, image_size: Vec2) -> Vec<YoloLabelBundle> {
//         let mut bundles: Vec<YoloLabelBundle> = Vec::new();
//         let yolo_file = self.0.clone();

//         for entry in yolo_file.entries.iter() {
//             let scaled_x_center = entry.x_center * image_size.x;
//             let scaled_y_center = entry.y_center * image_size.y;
//             let scaled_width = entry.width * image_size.x;
//             let scaled_height = entry.height * image_size.y; // Shape::Rectangle(Rectangle::new(80., 80.))

//             // Create a 2d rectangle
//             let spatial_rectangle_bundle = YoloLabelBundle {
//                 spatial_bundle: SpatialBundle {
//                     transform: Transform::from_translation(Vec3::new(
//                         scaled_x_center,
//                         scaled_y_center,
//                         0.,
//                     )),
//                     visibility: Visibility::Visible,
//                     ..Default::default()
//                 },
//                 rectangle: shapes::Rectangle {
//                     extents: Vec2::new(scaled_width, scaled_height),
//                     origin: shapes::RectangleOrigin::Center,
//                 },
//             };

//             bundles.push(spatial_rectangle_bundle);
//         }

//         bundles
//     }
// }

// WILO: Trying to load image, get size, and then draw bounding boxes
//       to scale on the image.

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    project_resource: Res<YoloProjectResource>,
) {
    let canvas_size = Extent3d {
        width: RESOLUTION_WIDTH as u32,
        height: RESOLUTION_HEIGHT as u32,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("canvas"),
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    canvas.resize(canvas_size);
    let canvas_handle = images.add(canvas);

    // commands.spawn((
    //     Name::new("canvas_camera"),
    //     Camera2dBundle {
    //         camera: Camera {
    //             // render before the "main pass" camera
    //             order: -1,
    //             target: RenderTarget::Image(canvas_handle.clone()),
    //             msaa_writeback: true,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     CanvasCamera,
    //     CANVAS_LAYER,
    // ));

    // Spawn the canvas
    let canvas_eid = commands
        .spawn((
            Name::new("canvas"),
            SpriteBundle {
                texture: canvas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    RESOLUTION_WIDTH / 2.0,
                    RESOLUTION_HEIGHT / 2.0,
                    0.0,
                )),
                ..Default::default()
            },
            Canvas,
        ))
        .id();

    // Load first image
    let valid_pairs = project_resource.0.get_valid_pairs();
    let pair = valid_pairs.first().unwrap();
    let image_handle: Handle<Image> = asset_server.load(pair.image_path.clone().unwrap());

    let image_eid = commands
        .spawn((
            Name::new("image"),
            SpriteBundle {
                texture: image_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    RESOLUTION_WIDTH / 2.0,
                    RESOLUTION_HEIGHT / 2.0,
                    1.0,
                )),
                ..Default::default()
            },
            ImageToLoad,
        ))
        .id();

    // Set the image as a child of the canvas
    commands.entity(canvas_eid).push_children(&[image_eid]);

    // Create a camera for the image
    commands.spawn((
        Name::new("image_camera"),
        Camera2dBundle {
            camera: Camera {
                // order: 0,
                target: RenderTarget::Image(canvas_handle.clone()),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                RESOLUTION_WIDTH / 2.0,
                RESOLUTION_HEIGHT / 2.0,
                100.0,
            )),
            ..Default::default()
        },
        CanvasCamera,
    ));
}

// pub fn load_labels_for_image_system(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     app_data: ResMut<AppData>,
//     project_resource: Res<YoloProjectResource>,
//     images: ResMut<Assets<Image>>,
//     query: Query<Entity, With<ImageToLoad>>,
// ) {
//     for entity in query.iter() {
//         println!("Loading labels for image");
//         let valid_pairs = project_resource.0.get_valid_pairs();
//         let pair = valid_pairs[app_data.index].clone();

//         let image_handle = asset_server
//             .get_handle::<Image>(pair.image_path.clone().unwrap())
//             .unwrap();

//         if let Some(image) = images.get(&image_handle) {
//             let image_size = Vec2::new(image.width() as f32, image.height() as f32);

//             let yolo_file = pair.label_file.unwrap();
//             let yolo_file = BevyYoloFile(yolo_file);

//             let bundles = yolo_file.to_rectangle(image_size);

//             for (index, bundle) in bundles.iter().enumerate() {
//                 println!("Spawning bundle");
//                 println!("Bundle position: {:#?}", bundle);

//                 let bundle = (
//                     Name::new(format!("label_{}", index)),
//                     ShapeBundle {
//                         path: GeometryBuilder::build_as(&bundle.rectangle),
//                         spatial: bundle.spatial_bundle.clone(),
//                         ..Default::default()
//                     },
//                     // Fill::color(REBECCA_PURPLE),
//                     Stroke::new(RED, 3.0),
//                 );

//                 commands.spawn(bundle);
//             }

//             // Remove ImageToLoad component
//             commands.entity(entity).remove::<ImageToLoad>();
//         }
//     }
// }

pub fn load_next_image_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_data: ResMut<AppData>,
    project_resource: Res<YoloProjectResource>,
    query: Query<Entity, With<ImageToLoad>>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        println!("Loading next image");

        // Remove ImageToLoad component
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
            ImageToLoad,
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

// Scales camera projection to fit the window (integer multiples only).
// fn fit_canvas_system(
//     mut resize_events: EventReader<WindowResized>,
//     mut projections: Query<&mut OrthographicProjection, With<CanvasCamera>>,
// ) {
//     for event in resize_events.read() {
//         let h_scale = event.width / RESOLUTION_WIDTH;
//         let v_scale = event.height / RESOLUTION_HEIGHT;
//         let mut projection = projections.single_mut();
//         projection.scale = 1. / h_scale.min(v_scale).round();
//     }
// }
