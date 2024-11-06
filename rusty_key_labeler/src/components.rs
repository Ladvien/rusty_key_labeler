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
}

#[derive(Component)]
pub struct ImageToLoad {
    pub path: String,
    pub yolo_file: YoloFile,
}

#[derive(Debug, Clone, Component)]
pub struct SelectedImage;

#[derive(Debug, Clone, Component)]
pub struct BoundingBox;
