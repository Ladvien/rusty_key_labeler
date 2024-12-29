use bevy::{
    asset::RenderAssetUsages,
    color::Color,
    image::Image,
    math::Vec2,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

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

pub fn color_to_float_array(color: Color) -> [f32; 4] {
    let r = color.to_linear().red;
    let g = color.to_linear().green;
    let b = color.to_linear().blue;
    let a = color.to_linear().alpha;

    [r, g, b, a]
}

pub fn create_image_from_color(color: Color, width: u32, height: u32) -> Image {
    let color_data = color_to_float_array(color);
    let pixel_data = color_data
        .into_iter()
        .flat_map(|channel| channel.to_ne_bytes())
        .collect::<Vec<_>>();

    // println!("Pixel data: {:#?}", pixel_data);

    Image::new_fill(
        Extent3d {
            width,
            height,
            ..Default::default()
        },
        TextureDimension::D2,
        &pixel_data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::MAIN_WORLD,
    )
}
