use bevy::prelude::Entity;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use yolo_io::YoloProject;
use yolo_io::YoloProjectConfig;

use crate::settings::Settings;

// #[derive(Resource, Debug, Clone)]
// pub struct YoloProjectResource(pub YoloProject);

#[derive(Resource, Debug, Clone)]
pub struct AppData {
    pub index: isize,
    pub ui_eid: Option<Entity>,
    pub yolo_project: YoloProject,
    pub config: Config,
    pub left_panel_eid: Option<Entity>,
}

#[derive(Debug, Serialize, Deserialize, Resource, Clone)]
pub struct Config {
    pub project_config: YoloProjectConfig,
    pub output_path: String,
    #[serde(default)]
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_config: YoloProjectConfig::default(),
            output_path: "output".to_string(),
            settings: Settings::default(),
        }
    }
}
