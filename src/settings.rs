use std::collections::HashMap;

use bevy::prelude::{KeyCode, Resource};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    pub r#type: String,
    pub project_name: String,
    pub folder_paths: FolderPaths,
    pub class_map: HashMap<u32, String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FolderPaths {
    pub train: String,
    pub validation: String,
    pub test: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct KeyMap {
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub pan_up: KeyCode,
    pub pan_down: KeyCode,
    pub pan_left: KeyCode,
    pub pan_right: KeyCode,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct PanFactor {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub zoom_factor: f32,
    pub pan_factor: PanFactor,
    pub key_map: KeyMap,
    pub output_format: OutputFormat,
}

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct Config {
    pub image_path: String,
    pub label_path: String,
    pub output_path: String,
    pub settings: Settings,
}
