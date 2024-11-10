use bevy::{color::Color, math::Vec2};

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
