mod bounding_boxes;
mod components;
mod resources;
mod settings;
mod systems;
mod utils;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_ui_views::BevyUiViewsPlugin;
use bevy_vector_shapes::Shape2dPlugin;
use bounding_boxes::BoundingBoxPainter;
use components::*;
use resources::*;
use systems::*;
use yolo_io::YoloProject;

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml
    let data =
        std::fs::read_to_string("rusty_key_labeler/config.yaml").expect("Unable to read file");
    let config: Config = serde_yml::from_str(&data).expect("Unable to parse YAML");
    let project = YoloProject::new(&config.project_config).expect("Unable to create project");

    // let report = YoloDataQualityReport::generate(project.clone().unwrap());

    // match report {
    //     Some(report) => {
    //         let mut file = fs::File::create("report.json").expect("Unable to create file");
    //         file.write_all(report.as_bytes())
    //             .expect("Unable to write data to file");
    //     }
    //     None => todo!(),
    // }

    // let project_resource = YoloProjectResource(project.unwrap());

    let bb_painter = BoundingBoxPainter::new(
        &config.settings.bounding_boxes,
        &config.project_config.export.class_map,
    );

    let app_data = AppData {
        index: 0,
        ui_eid: None,
        yolo_project: project,
        config: config.clone(),
        left_panel_eid: None,
    };

    // let font = "RobotoMono-Regular.ttf";
    // let font_size = 16.0;
    let ui = Ui::new(
        &config.settings.ui_panel.colors,
        config.settings.ui_panel.font_size,
        &config.settings.ui_panel.font_path,
    );

    App::new()
        .init_resource::<Assets<ColorMaterial>>()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // Makes images crisp
            WorldInspectorPlugin::new(),
            Shape2dPlugin::default(),
            // UiDefaultPlugins,
            BevyUiViewsPlugin,
        ))
        .insert_resource(bb_painter)
        .insert_resource(app_data)
        .insert_resource(ui)
        .add_systems(Startup, (setup, ui_setup))
        .add_systems(
            Update,
            (
                next_and_previous_system,
                bounding_boxes_system,
                update_labeling_index,
                update_current_file_name_label,
                debounce_timer_system,
                image_state_system,
                debug_viewport,
                translate_image_system,
                compute_canvas_viewport_data,
                zoom_image_system, // center_image_on_load,
            )
                .chain(),
        )
        .run();
}
