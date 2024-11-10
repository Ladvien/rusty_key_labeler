use bevy::{
    color::Color,
    core::Name,
    math::{Vec2, Vec3},
    prelude::{Commands, Component, Res, Resource, Transform},
    render::view::RenderLayers,
    sprite::{Sprite, SpriteBundle},
    window::Window,
};

use crate::settings::{TopLeftPosition, UiPanelSettings, UiPanelSize, UI_LAYER};

#[derive(Debug, Component, Clone)]
pub struct UiPanel;

#[derive(Debug, Clone, Component)]
pub struct UiDataChanged;

#[derive(Debug, Resource, Clone)]
pub struct UI {
    size: UiPanelSize,
    original_top_left_position: TopLeftPosition,
    top_left_position: TopLeftPosition,
    color: Color,
}

impl UI {
    pub fn new(ui_data: UiPanelSettings) -> Self {
        Self {
            size: ui_data.size,
            original_top_left_position: ui_data.top_left_position.clone(),
            top_left_position: ui_data.top_left_position,
            color: ui_data.color,
        }
    }

    // pub fn update(&mut self, ui_data: UiPanelSettings, window: &Window) {
    //     self.top_left_position =
    //         self.get_ui_window_xy(&ui_data.top_left_position, &ui_data.size, window);
    //     self.size = ui_data.size;
    //     self.color = ui_data.color;
    // }

    // pub fn update_scale(&mut self, ui_data: UiPanelSettings, window: &Window) {}

    pub fn on_window_resize(&mut self, mut commands: Commands, window: &Window) {
        self.top_left_position =
            self.get_ui_window_xy(&self.original_top_left_position, &self.size, window);
        commands.spawn(UiDataChanged);
    }

    pub fn to_transform(&self) -> Transform {
        // WILO: This seems to be producing outrageous values.
        let translation = Vec3::new(
            self.top_left_position.x as f32,
            self.top_left_position.y as f32,
            // TODO: Set proper z-index.
            99.,
        );
        Transform::from_translation(translation)
    }

    pub fn get_ui_bundle(&self) -> (Name, SpriteBundle, UiPanel, RenderLayers) {
        let transform = self.to_transform();
        let bundle = (
            Name::new("ui"),
            SpriteBundle {
                sprite: Sprite {
                    color: self.color,
                    custom_size: Some(Vec2::new(self.size.width, self.size.height)),
                    ..Default::default()
                },
                transform,
                ..Default::default()
            },
            UiPanel,
            UI_LAYER,
        );

        bundle
    }

    // Privates
    fn get_ui_window_xy(
        &self,
        origin: &TopLeftPosition,
        box_size: &UiPanelSize,
        window: &Window,
    ) -> TopLeftPosition {
        println!("TopLeftPosition: {:#?}", origin);
        println!(
            "Window: width {:#?}, height {:#?}",
            window.width(),
            window.height()
        );

        let half_width = window.width() / 2.;
        let half_height = window.height() / 2.;
        let half_box_width = box_size.width / 2.;
        let half_box_height = box_size.height / 2.;

        let x = (origin.x as f32 - half_width + half_box_width) as i32;
        let y = (origin.y as f32 - half_height + half_box_height) as i32;

        let top_left_position = TopLeftPosition { x, y };

        top_left_position
    }
}
