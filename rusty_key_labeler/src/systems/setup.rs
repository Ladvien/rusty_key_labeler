use crate::{
    resources::AppData, utils::create_image_from_color, ImageLoading, ImageViewport, MainCamera,
    Ui, ViewportCamera,
};
use bevy::{
    prelude::*,
    render::camera::{RenderTarget, Viewport},
};

use super::VIEWPORT_LAYER;

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    asset_server: Res<AssetServer>,
    mut ui: ResMut<Ui>,
    mut images: ResMut<Assets<Image>>,
) {
    app_data.index = 0;
    let valid_pairs = app_data.yolo_project.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let _ = asset_server.load::<Image>(first_image_path.clone());

    let canvas_image = create_image_from_color(Color::BLACK, 1000, 1000);
    let canvas_image_handle = images.add(canvas_image);

    commands.spawn((
        Name::new("viewport_camera"),
        Camera2d,
        ViewportCamera,
        Camera {
            target: RenderTarget::Image(canvas_image_handle.clone()),
            ..Default::default()
        },
        Msaa::Off,
        VIEWPORT_LAYER,
    ));

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2d,
        Camera {
            order: 1,
            ..Default::default()
        },
        Msaa::Off,
        // Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        // BloomSettings {
        //     intensity: 0.1,
        //     high_pass_frequency: 0.1,
        //     ..Default::default()
        // },
        MainCamera,
    ));

    let valid_pairs = app_data.yolo_project.get_valid_pairs();

    // Load next image
    let next_image_path = match valid_pairs[app_data.index as usize].image_path.clone() {
        Some(image) => image,
        _ => {
            error!("Image path not found");
            return;
        }
    };
    let next_image = next_image_path.as_path().to_string_lossy().into_owned();

    // Add image to the scene
    let next_image_handle = asset_server.load::<Image>(next_image.clone());

    commands.spawn(ImageLoading(next_image_handle.clone()));

    let font_handle: Handle<Font> = asset_server.load(ui.font_path.clone());
    ui.font_handle = Some(font_handle.clone());

    let (container_ui_eid, left_panel_ui_eid) = ui.spawn_ui(&mut commands, canvas_image_handle);

    app_data.ui_eid = Some(container_ui_eid);
    app_data.left_panel_eid = Some(left_panel_ui_eid);
}
