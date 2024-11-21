use bevy::prelude::Entity;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use yolo_io::YoloProject;
use yolo_io::YoloProjectConfig;

use crate::settings::Settings;

#[derive(Resource, Debug, Clone)]
pub struct YoloProjectResource(pub YoloProject);

#[derive(Resource, Debug, Clone)]
pub struct AppData {
    pub index: isize,
    pub total_images: isize,
    pub ui_eid: Option<Entity>,
}

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct Config {
    pub project_config: YoloProjectConfig,
    pub output_path: String,
    #[serde(default)]
    pub settings: Settings,
}
