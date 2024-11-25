mod bounding_boxes;
mod components;
mod resources;
mod settings;
mod systems;
mod ui;
mod utils;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_lunex::{prelude::MainUi, UiDebugPlugin, UiDefaultPlugins};
use bevy_ui_views::BevyUiViewsPlugin;
use bevy_vector_shapes::prelude::*;

use bounding_boxes::BoundingBoxPainter;
use components::*;
use resources::*;
use systems::*;
use ui::UI;
use yolo_io::YoloProject;

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

    let ui = UI::new(config.settings.ui_panel.clone());

    let bb_painter = BoundingBoxPainter::new(
        &config.settings.bounding_boxes,
        &config.project_config.export.class_map,
    );

    let num_valid_images = project_resource.0.get_valid_pairs().len() as isize;

    let app_data = AppData {
        index: 0,
        total_images: num_valid_images - 1,
        ui_eid: None,
    };

    App::new()
        .init_resource::<Assets<ColorMaterial>>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            // WorldInspectorPlugin::new(),
            Shape2dPlugin::default(),
            UiDefaultPlugins,
            BevyUiViewsPlugin,
            // UiDebugPlugin::<MainUi>::new(),
        ))
        .insert_resource(config)
        .insert_resource(ui)
        .insert_resource(bb_painter)
        .insert_resource(project_resource)
        .insert_resource(app_data)
        .add_systems(
            Startup,
            (
                setup,
                // setup_ui
            ),
        )
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
