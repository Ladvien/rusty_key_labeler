use bevy::prelude::*;
use bevy::{
    color::{Color, Srgba},
    math::{Vec2, Vec3},
    prelude::Transform,
    utils::HashMap,
};

use crate::{Config, YoloProjectResource};

pub fn srgba_string_to_color(srgba_string: &str) -> Option<Color> {
    let rgba: Vec<&str> = srgba_string
        .trim_matches(|p| p == '(' || p == ')')
        .split(',')
        .map(|s| s.trim())
        .collect();

    let red = rgba[0].parse::<u8>().ok()?;
    let green = rgba[1].parse::<u8>().ok()?;
    let blue = rgba[2].parse::<u8>().ok()?;
    let alpha = rgba[3].parse::<u8>().ok()?;

    Some(Color::srgba_u8(red, green, blue, alpha))
}

pub fn get_bounding_box_transform(x_center: f32, y_center: f32, image_size: Vec2) -> Transform {
    Transform::from_translation(Vec3::new(
        x_center - image_size.x / 2.,
        (y_center - image_size.y / 2.) * -1.,
        0.,
    ))
}

pub fn scale_dimensions(
    x_center: f32,
    y_center: f32,
    width: f32,
    height: f32,
    image_size: Vec2,
) -> (f32, f32, f32, f32) {
    let scaled_x_center = x_center * image_size.x;
    let scaled_y_center = y_center * image_size.y;
    let scaled_width = width * image_size.x;
    let scaled_height = height * image_size.y;

    (
        scaled_x_center,
        scaled_y_center,
        scaled_width,
        scaled_height,
    )
}

pub fn get_class_color_map(project_resource: &YoloProjectResource) -> HashMap<String, String> {
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
        let color = colors[*i];
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
