use bevy::prelude::*;

use super::start_image_load;
use crate::{
    resources::AppData,
    settings::{MAIN_LAYER, UI_LAYER},
    utils::create_image_from_color,
    MainCamera, Ui, UiCamera, UninitializedRenderTarget,
};

pub fn setup(
    mut commands: Commands,
    mut app_data: ResMut<AppData>,
    mut ui: ResMut<Ui>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    app_data.index = 0;
    let valid_pairs = app_data.yolo_project.get_valid_pairs();
    let selected_pair = valid_pairs[0].clone();

    let first_image = selected_pair.clone().image_path.unwrap();
    let first_image_path = first_image.as_path().to_string_lossy().into_owned();
    let _ = asset_server.load::<Image>(first_image_path.clone());

    let canvas_image =
        create_image_from_color(Color::from(Srgba::new(0.0, 0.0, 0.0, 0.0)), 2024, 1268);
    let canvas_image_handle = images.add(canvas_image);

    // Load camera
    commands.spawn((
        Name::new("main_camera"),
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        MAIN_LAYER,
        MainCamera,
    ));

    commands.spawn((UninitializedRenderTarget,));

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

    let (container_ui_eid, left_panel_ui_eid) = ui.spawn_ui(&mut commands, &canvas_image_handle);

    app_data.ui_eid = Some(container_ui_eid);
    app_data.left_panel_eid = Some(left_panel_ui_eid);

    start_image_load(
        &mut commands,
        asset_server,
        app_data.index,
        valid_pairs.len() as isize - 1,
        0.0,
        valid_pairs,
    );
}
