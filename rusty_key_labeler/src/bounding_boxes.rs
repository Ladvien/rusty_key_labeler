use bevy::{
    color::{Alpha, Srgba},
    core::Name,
    math::{Vec2, Vec3, Vec4},
    prelude::{Component, Resource, Transform},
    render::view::RenderLayers,
};

use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{RectangleBundle, RectangleComponent, ShapeBundle},
};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use yolo_io::YoloFile;

use crate::{
    settings::MAIN_LAYER,
    utils::{scale_dimensions, srgba_string_to_color},
    Config, YoloProjectResource,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundingBoxSettings {
    pub thickness: f32,
    pub corner_radius: f32,
    pub class_color_map: HashMap<String, String>,
}

impl Default for BoundingBoxSettings {
    fn default() -> Self {
        Self {
            thickness: 1.0,
            corner_radius: 0.3,
            class_color_map: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundingBoxEntry {
    pub class: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, PartialEq, Clone, Resource)]
pub struct BoundingBoxPainter {
    pub bounding_box_settings: BoundingBoxSettings,
    pub class_map: HashMap<isize, String>,
}

#[derive(Debug, PartialEq, Clone, Component)]
pub struct BoundingBoxMarker;

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

    pub fn get_boxes(
        &self,
        yolo_file: &YoloFile,
        image_size: Vec2,
    ) -> Vec<(
        Name,
        ShapeBundle<RectangleComponent>,
        BoundingBoxMarker,
        RenderLayers,
    )> {
        let mut bounding_boxes = vec![];

        for (index, entry) in yolo_file.entries.iter().enumerate() {
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

            let class_names_map = self.class_map.clone();
            let class_name = class_names_map[&entry.class].clone();
            let class_color_string =
                self.bounding_box_settings.class_color_map[&class_name].clone();

            if let Some(class_color) = srgba_string_to_color(&class_color_string) {
                let bounding_box = (
                    Name::new(format!("bounding_box_{}", index)),
                    ShapeBundle::rect(
                        &ShapeConfig {
                            color: class_color,
                            transform: bounding_box_transform,
                            hollow: true,
                            thickness: self.bounding_box_settings.thickness,
                            corner_radii: Vec4::splat(self.bounding_box_settings.corner_radius),
                            ..ShapeConfig::default_2d()
                        },
                        size,
                    ),
                    BoundingBoxMarker,
                    MAIN_LAYER,
                );

                bounding_boxes.push(bounding_box);
            }
        }

        // children
        bounding_boxes
    }

    fn get_bounding_box_transform(x_center: f32, y_center: f32, image_size: Vec2) -> Transform {
        Transform::from_translation(Vec3::new(
            x_center - image_size.x / 2.,
            (y_center - image_size.y / 2.) * -1.,
            0.,
        ))
    }

    fn get_class_color_map(project_resource: &YoloProjectResource) -> HashMap<String, String> {
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

        // 2. Generate a color for each class.
        let class_map = project_resource.0.config.export.class_map.clone();

        // 3. Assign the color to the class.
        let mut class_color_map: HashMap<String, String> = HashMap::new();
        for (i, class_name) in class_map.iter() {
            let color = colors[*i as usize];
            let rgba_color_string = format!(
                "rgba({}, {}, {}, {})",
                color.red,
                color.green,
                color.blue,
                color.alpha()
            );

            class_color_map.insert(class_name.to_owned(), rgba_color_string);
        }

        class_color_map
    }

    // pub fn get_entries(&self) -> Vec<BoundingBoxEntry> {}
}
