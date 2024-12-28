use super::start_image_load;
use crate::{
    resources::AppData,
    settings::{MAIN_LAYER, UI_LAYER},
    MainCamera, Ui, UiCamera,
};
use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    asset_server: Res<AssetServer>,
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
        MAIN_LAYER,
        MainCamera,
    ));

    start_image_load(
        &mut commands,
        asset_server,
        app_data.index,
        valid_pairs.len() as isize - 1,
        0.0,
        valid_pairs,
    );
}

// Systems on Setup
pub fn ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_data: ResMut<AppData>,
    mut ui: ResMut<Ui>,
) {
    let font_handle: Handle<Font> = asset_server.load(ui.font_path.clone());
    ui.font_handle = Some(font_handle.clone());

    commands.spawn((
        Name::new("ui_camera"),
        Camera2d,
        Camera {
            // Render the UI on top of everything else.
            order: 1,
            ..default()
        },
        UI_LAYER,
        UiCamera,
    ));

    let (container_ui_eid, left_panel_ui_eid) = ui.spawn_ui(&mut commands);

    app_data.ui_eid = Some(container_ui_eid);
    app_data.left_panel_eid = Some(left_panel_ui_eid);
}
