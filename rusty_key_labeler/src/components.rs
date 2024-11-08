use bevy::prelude::*;
use yolo_io::YoloFile;

use crate::settings::{TopLeftPosition, UiPanelSize};

// Create a generic component to flag
// changed entities.
// #[derive(Debug, Clone, Component)]
// pub struct Updated<T: Component + Send + Sync>(pub T);

// impl<T: Component + Send + Sync> Updated<T> {
//     pub fn new(data: T) -> Self {
//         Self(data)
//     }
// }

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

#[derive(Debug, Clone, Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Component)]
pub struct UiCamera;
