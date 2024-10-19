use bevy::prelude::{KeyCode, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yolo_io::YoloProjectConfig;

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
}

#[derive(Debug, Serialize, Deserialize, Resource)]
pub struct Config {
    pub project_config: YoloProjectConfig,
    pub output_path: String,
    pub output_format: OutputFormat,
    #[serde(default)]
    pub settings: Settings,
}

impl Default for FolderPaths {
    fn default() -> Self {
        Self {
            train: "train".to_string(),
            validation: "validation".to_string(),
            test: "test".to_string(),
        }
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            zoom_in: KeyCode::Equal,
            zoom_out: KeyCode::Minus,
            pan_up: KeyCode::KeyW,
            pan_down: KeyCode::KeyS,
            pan_left: KeyCode::KeyA,
            pan_right: KeyCode::KeyD,
        }
    }
}

impl Default for PanFactor {
    fn default() -> Self {
        Self { x: 120.0, y: 120.0 }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zoom_factor: 1.0,
            pan_factor: PanFactor::default(),
            key_map: KeyMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_folder_paths() {
        let folder_paths = FolderPaths::default();
        assert_eq!(
            folder_paths,
            FolderPaths {
                train: "train".to_string(),
                validation: "validation".to_string(),
                test: "test".to_string()
            }
        );
    }

    #[test]
    fn test_default_key_map() {
        let key_map = KeyMap::default();
        assert_eq!(
            key_map,
            KeyMap {
                zoom_in: KeyCode::Equal,
                zoom_out: KeyCode::Minus,
                pan_up: KeyCode::KeyW,
                pan_down: KeyCode::KeyS,
                pan_left: KeyCode::KeyA,
                pan_right: KeyCode::KeyD
            }
        );
    }

    #[test]
    fn test_default_pan_factor() {
        let pan_factor = PanFactor::default();
        assert_eq!(pan_factor, PanFactor { x: 120.0, y: 120.0 });
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(
            settings,
            Settings {
                zoom_factor: 1.0,
                pan_factor: PanFactor::default(),
                key_map: KeyMap::default()
            }
        );
    }
}
