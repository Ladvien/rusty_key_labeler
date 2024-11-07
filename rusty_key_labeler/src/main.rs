mod components;
mod resources;
mod settings;
mod systems;
mod ui;
mod utils;

use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_vector_shapes::prelude::*;

use components::*;
use resources::*;
use systems::*;
use ui::{UiData, UI};
use utils::get_class_color_map;
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
        config.settings.bounding_boxes.class_color_map = get_class_color_map(&project_resource);
    }

    let ui_data = UiData {
        size: config.settings.ui_panel.size.clone(),
        top_left_position: config.settings.ui_panel.top_left_position.clone(),
        color: config.settings.ui_panel.color,
    };
    let ui = UI::new(ui_data);

    let app_data = AppData { index: 0 };

    App::new()
        .init_resource::<Assets<ColorMaterial>>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WorldInspectorPlugin::new(),
            Shape2dPlugin::default(),
        ))
        .insert_resource(config)
        .insert_resource(ui)
        .insert_resource(project_resource)
        .insert_resource(app_data)
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            (
                zoom_system,
                translate_image_system,
                next_and_previous_system,
                paint_bounding_boxes_system,
                on_image_loaded_system,
                on_resize_system,
                update_ui_panel,
            ),
        )
        .run();
}
