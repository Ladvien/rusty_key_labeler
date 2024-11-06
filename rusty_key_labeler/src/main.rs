mod components;
mod resources;
mod settings;
mod systems;

use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_vector_shapes::prelude::*;

use components::*;
use resources::*;
use systems::*;
use yolo_io::YoloProject;

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data =
        std::fs::read_to_string("rusty_key_labeler/config.yaml").expect("Unable to read file");
    let mut config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");
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

    let color_map = config.settings.bounding_boxes.class_color_map.clone();

    if color_map.len() == 0 {
        // 1. Determine how many classes are in the project.
        let num_classes = project_resource.0.data.number_of_classes;
        let colors = vec![
            Srgba::rgba_u8(255, 0, 0, 255),     // Red
            Srgba::rgba_u8(0, 255, 0, 255),     // Green
            Srgba::rgba_u8(0, 0, 255, 255),     // Blue
            Srgba::rgba_u8(255, 255, 0, 255),   // Yellow
            Srgba::rgba_u8(128, 0, 128, 255),   // Purple
            Srgba::rgba_u8(255, 165, 0, 255),   // Orange
            Srgba::rgba_u8(255, 192, 203, 255), // Pink
            Srgba::rgba_u8(165, 42, 42, 255),   // Brown
            Srgba::rgba_u8(128, 128, 128, 255), // Gray
            Srgba::rgba_u8(0, 255, 255, 255),   // Cyan
            Srgba::rgba_u8(0, 255, 0, 255),     // Lime
            Srgba::rgba_u8(0, 128, 128, 255),   // Teal
            Srgba::rgba_u8(75, 0, 130, 255),    // Indigo
            Srgba::rgba_u8(255, 191, 0, 255),   // Amber
            Srgba::rgba_u8(255, 87, 34, 255),   // Deep Orange
            Srgba::rgba_u8(103, 58, 183, 255),  // Deep Purple
            Srgba::rgba_u8(3, 169, 244, 255),   // Light Blue
            Srgba::rgba_u8(139, 195, 74, 255),  // Light Green
            Srgba::rgba_u8(96, 125, 139, 255),  // Blue Gray
        ];
        // 2. Generate a color for each class.
        let class_map = project_resource.0.config.export.class_map.clone();
        // 3. Assign the color to the class.
        let mut class_color_map: HashMap<String, String> = HashMap::new();
        for (i, class_name) in class_map.iter() {
            let color = colors[*i];
            let rgba_color_string = format!(
                "rgba({}, {}, {}, {})",
                color.red,
                color.green,
                color.blue,
                color.alpha()
            );

            class_color_map.insert(class_name.to_owned(), rgba_color_string);
        }
        // 4. Assign the color map to the bounding boxes settings.
        config.settings.bounding_boxes.class_color_map = class_color_map;
    }

    let app_data = AppData { index: 0 };

    App::new()
        .init_resource::<Assets<ColorMaterial>>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::new(),
            Shape2dPlugin::default(),
        ))
        .insert_resource(config)
        .insert_resource(project_resource)
        .insert_resource(app_data)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                zoom_system,
                translate_image_system,
                next_and_previous_system,
                paint_bounding_boxes_system,
                on_image_loaded_system,
            ),
        )
        .run();
}
