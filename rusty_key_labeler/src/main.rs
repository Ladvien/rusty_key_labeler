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

struct AppInputs {
    pub config: Config,
    pub settings: String,
    pub bounding_box_painter: BoundingBoxPainter,
    pub ui: Ui,
    pub app_data: AppData,
}

fn prepare_app_inputs(path: &str) -> Result<AppInputs, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    let config: Config = serde_yml::from_str(&data)?;
    let project = YoloProject::new(&config.project_config)?;

    let bb_painter = BoundingBoxPainter::new(
        &config.settings.bounding_boxes,
        &config.project_config.export.class_map,
    );

    let ui = Ui::new(
        &config.settings.ui_panel.colors,
        config.settings.ui_panel.font_size,
        &config.settings.ui_panel.font_path,
    );

    let app_data = AppData {
        index: 0,
        ui_eid: None,
        yolo_project: project,
        config: config.clone(),
        left_panel_eid: None,
    };

    Ok(AppInputs {
        config,
        settings: data,
        bounding_box_painter: bb_painter,
        ui,
        app_data,
    })
}

fn main() {
    // Load YAML configuration file from file.
    // https://github.com/sebastienrousseau/serde_yml

    match prepare_app_inputs("rusty_key_labeler/config.yaml") {
        Ok(app_inputs) => {
            App::new()
                .init_resource::<Assets<ColorMaterial>>()
                .add_plugins((
                    DefaultPlugins.set(ImagePlugin::default_nearest()), // Makes images crisp
                    WorldInspectorPlugin::new(),
                    Shape2dPlugin::default(),
                    BevyUiViewsPlugin,
                ))
                .insert_resource(app_inputs.bounding_box_painter)
                .insert_resource(app_inputs.app_data)
                .insert_resource(app_inputs.ui)
                .add_systems(Startup, (setup,))
                .add_systems(
                    Update,
                    (
                        image_selection_system,
                        load_bounding_boxes,
                        update_labeling_index,
                        update_current_file_name_label,
                        debounce_timer_system,
                        image_state_system,
                        translate_image_system,
                        zoom_image_system,
                        fit_to_viewport,
                        center_in_viewport,
                        compute_viewport,
                        cycle_bounding_box_selection,
                        highlight_bounding_box,
                        select_bounding_box_nearest_center,
                    )
                        .chain(),
                )
                .run();
        }
        Err(e) => {
            // TODO: Add error app here.
        }
    }
}
