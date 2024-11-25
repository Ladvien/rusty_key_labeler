use bevy::{color::Color, prelude::KeyCode, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{bounding_boxes::BoundingBoxSettings, utils::srgba_string_to_color};

pub const MAIN_LAYER: RenderLayers = RenderLayers::layer(0);
pub const UI_LAYER: RenderLayers = RenderLayers::layer(1);

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
pub struct TopLeftPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UiPanelSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct UiPanelSettings {
    pub top_left_position: TopLeftPosition,
    pub color: Color,
    #[serde(rename = "size")]
    pub size: UiPanelSize,
}

impl<'de> Deserialize<'de> for UiPanelSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct UiPanelSettingsHelper {
            size: Option<UiPanelSize>,
            color: Option<String>,
            top_left_position: Option<TopLeftPosition>,
        }

        let helper = UiPanelSettingsHelper::deserialize(deserializer)?;
        let color =
            srgba_string_to_color(&helper.color.unwrap_or("rgba(0, 0, 128, 128)".to_string()))
                .ok_or_else(|| serde::de::Error::custom("Invalid color"))?;

        let ui_panel_settings = UiPanelSettings {
            size: helper.size.unwrap_or(UiPanelSize {
                width: 0.2,
                height: 0.15,
            }),
            color,
            top_left_position: helper
                .top_left_position
                .unwrap_or(TopLeftPosition { x: 0, y: 0 }),
        };

        Ok(ui_panel_settings)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub zoom_factor: f32,
    pub pan_factor: PanFactor,
    pub key_map: KeyMap,
    pub bounding_boxes: BoundingBoxSettings,
    pub ui_panel: UiPanelSettings,
    pub delay_between_images: f32,
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
            bounding_boxes: BoundingBoxSettings::default(),
            ui_panel: UiPanelSettings {
                size: UiPanelSize {
                    width: 0.2,
                    height: 0.15,
                },
                color: Color::srgba_u8(0, 0, 128, 128),
                top_left_position: TopLeftPosition { x: 0, y: 0 },
            },
            delay_between_images: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use yolo_io::Paths;

    use super::*;

    #[test]
    fn test_default_folder_paths() {
        let folder_paths = Paths::default();
        assert_eq!(
            folder_paths,
            Paths {
                train: "train".to_string(),
                validation: "validation".to_string(),
                test: "test".to_string(),
                root: "yolo".to_string()
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
                key_map: KeyMap::default(),
                bounding_boxes: BoundingBoxSettings::default(),
                ui_panel: UiPanelSettings {
                    color: Color::srgba_u8(0, 0, 128, 128),
                    top_left_position: TopLeftPosition { x: 0, y: 0 },
                    size: UiPanelSize {
                        width: 0.2,
                        height: 0.15
                    }
                },
                delay_between_images: 0.1,
            }
        );
    }
}
