use bevy::prelude::*;
use yolo_io::YoloFile;

#[derive(Debug, Clone, Component)]
pub struct ImageData {
    pub path: String,
    pub stem: String,
    pub image: Handle<Image>,
    pub width: f32,
    pub height: f32,
    pub yolo_file: YoloFile,
    pub index: isize,
    pub total_images: isize,
}

#[derive(Component)]
pub struct ImageToLoad {
    pub path: String,
    pub yolo_file: YoloFile,
}

#[derive(Debug, Clone, Component)]
pub struct ImageBeingLoaded;

#[derive(Debug, Clone, Component)]
pub struct SelectedImage;

#[derive(Debug, Clone, Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Component)]
pub struct UiCamera;

#[derive(Debug, Clone, Component)]
pub struct UiData {
    pub stem: String,
    pub image_path: String,
    pub label_path: String,
}

#[derive(Debug, Clone, Component)]
pub struct DebounceTimer {
    pub timer: Timer,
}

#[derive(Debug, Clone, Component)]
pub struct UILeftPanel;

#[derive(Debug, Clone, Component)]
pub struct UIBottomPanel;
