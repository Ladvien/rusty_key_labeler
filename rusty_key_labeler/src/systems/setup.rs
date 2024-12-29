use crate::{resources::AppData, ImageLoading, MainCamera, TopRightPanelUI, Ui, UiCamera};
use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    asset_server: Res<AssetServer>,
    mut canvas: Query<&mut ImageNode, With<TopRightPanelUI>>,
) {
    app_data.index = 0;
    let valid_pairs = app_data.yolo_project.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let _ = asset_server.load::<Image>(first_image_path.clone());

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2d,
        Transform::from_xyz(0., 0., 16.).looking_at(Vec3::ZERO, Vec3::Y),
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

    commands.spawn((ImageLoading(next_image_handle.clone()),));
}
