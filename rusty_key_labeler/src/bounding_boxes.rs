use std::process::Command;

use bevy::{
    color::{Color, Srgba},
    core::Name,
    math::{Vec2, Vec3, Vec4},
    prelude::{Commands, Component, Entity, Resource, Transform},
    render::view::RenderLayers,
};
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{DiscBundle, RectangleBundle, RectangleComponent, ShapeBundle},
};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    settings::MAIN_LAYER,
    utils::{scale_dimensions, srgba_string_to_color},
    CornerHandle,
};
use yolo_io::YoloEntry;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct BoundingBoxSettings {
    pub handle_size: f32,
    pub thickness: f32,
    pub corner_radius: f32,
    #[serde(default)]
    pub class_color_map: Vec<Srgba>,
}

impl Default for BoundingBoxSettings {
    fn default() -> Self {
        Self {
            handle_size: 7.5,
            thickness: 1.0,
            corner_radius: 0.3,
            class_color_map: get_class_color_map(),
        }
    }
}

impl<'de> Deserialize<'de> for BoundingBoxSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct BoundingBoxSettingsHelper {
            handle_size: Option<f32>,
            thickness: Option<f32>,
            corner_radius: Option<f32>,
            class_color_map: Option<Vec<String>>,
        }

        let helper = BoundingBoxSettingsHelper::deserialize(deserializer)?;
        let handle_size = helper.handle_size.unwrap_or(7.5);
        let thickness = helper.thickness.unwrap_or(1.0);
        let corner_radius = helper.corner_radius.unwrap_or(0.3);
        let class_color_map = helper
            .class_color_map
            .unwrap_or_else(|| vec![])
            .iter()
            .map(|color| srgba_string_to_color(color).map(|c| c.into()))
            .collect::<Option<Vec<Srgba>>>()
            .ok_or_else(|| serde::de::Error::custom("Invalid color"))?;

        Ok(Self {
            handle_size,
            thickness,
            corner_radius,
            class_color_map,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Component)]
pub struct BoundingBox {
    pub index: usize,
    pub class: String,
    pub class_color: Color,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PartialOrd for BoundingBox {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let order_value = (self.width * 1000.0 + self.height * 1000.0) as usize;
        let other_order_value = (other.width * 1000.0 + other.height * 1000.0) as usize;
        Some(order_value.cmp(&other_order_value))
    }
}

#[derive(Debug, PartialEq, Clone, Component)]
pub struct SelectedBoundingBox;

#[derive(Debug, PartialEq, Clone, Resource)]
pub struct BoundingBoxPainter {
    pub bounding_box_settings: BoundingBoxSettings,
    pub class_map: HashMap<isize, String>,
}

#[derive(Debug, PartialEq, Clone, Component)]
pub struct ContainsBoundingBoxes;

impl BoundingBoxPainter {
    pub fn new(
        bounding_box_settings: &BoundingBoxSettings,
        class_map: &HashMap<isize, String>,
    ) -> Self {
        Self {
            bounding_box_settings: bounding_box_settings.clone(),
            class_map: class_map.clone(),
        }
    }

    pub fn get_color(&self, class: isize) -> Color {
        let class_color = self.bounding_box_settings.class_color_map[class as usize];
        Color::from(class_color)
    }

    pub fn spawn_bounding_box(
        &self,
        commands: &mut Commands,
        index: usize,
        entry: &YoloEntry,
        image_size: Vec2,
    ) -> Entity {
        let (scaled_x_center, scaled_y_center, scaled_width, scaled_height) = scale_dimensions(
            entry.x_center,
            entry.y_center,
            entry.width,
            entry.height,
            image_size,
        );

        let bounding_box_transform =
            Self::get_bounding_box_transform(scaled_x_center, scaled_y_center, image_size);

        let size = Vec2::new(scaled_width, scaled_height);

        let class_color = self.bounding_box_settings.class_color_map[entry.class as usize];

        let bounding_box_eid = &commands
            .spawn((
                Name::new(format!("bounding_box_{}", index)),
                ShapeBundle::rect(
                    &ShapeConfig {
                        color: Color::from(class_color),
                        transform: bounding_box_transform,
                        hollow: true,
                        thickness: self.bounding_box_settings.thickness,
                        corner_radii: Vec4::splat(self.bounding_box_settings.corner_radius),
                        ..ShapeConfig::default_2d()
                    },
                    size,
                ),
                BoundingBox {
                    index,
                    class: self.class_map[&entry.class].clone(),
                    class_color: Color::from(class_color),
                    x: scaled_x_center,
                    y: scaled_y_center,
                    width: scaled_width,
                    height: scaled_height,
                },
                MAIN_LAYER,
            ))
            .id();

        *bounding_box_eid
    }

    fn get_bounding_box_transform(x_center: f32, y_center: f32, image_size: Vec2) -> Transform {
        Transform::from_translation(Vec3::new(
            x_center - image_size.x / 2.,
            (y_center - image_size.y / 2.) * -1.,
            0.,
        ))
    }
}

fn get_class_color_map() -> Vec<Srgba> {
    let colors = vec![
        Srgba::rgba_u8(255, 0, 0, 255),     // Red
        Srgba::rgba_u8(0, 255, 0, 255),     // Green
        Srgba::rgba_u8(0, 0, 255, 255),     // Blue
        Srgba::rgba_u8(255, 255, 0, 255),   // Yellow
        Srgba::rgba_u8(128, 0, 128, 255),   // Purple
        Srgba::rgba_u8(255, 165, 0, 255),   // Orange
        Srgba::rgba_u8(255, 192, 203, 255), // Pink
        Srgba::rgba_u8(165, 42, 42, 255),   // Brown
        Srgba::rgba_u8(128, 128, 128, 255), // Gray
        Srgba::rgba_u8(0, 255, 255, 255),   // Cyan
        Srgba::rgba_u8(0, 255, 0, 255),     // Lime
        Srgba::rgba_u8(0, 128, 128, 255),   // Teal
        Srgba::rgba_u8(75, 0, 130, 255),    // Indigo
        Srgba::rgba_u8(255, 191, 0, 255),   // Amber
        Srgba::rgba_u8(255, 87, 34, 255),   // Deep Orange
        Srgba::rgba_u8(103, 58, 183, 255),  // Deep Purple
        Srgba::rgba_u8(3, 169, 244, 255),   // Light Blue
        Srgba::rgba_u8(139, 195, 74, 255),  // Light Green
        Srgba::rgba_u8(96, 125, 139, 255),  // Blue Gray
    ];

    colors
}
