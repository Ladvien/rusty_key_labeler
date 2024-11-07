use bevy::{
    color::Color,
    math::{Vec2, Vec3},
    prelude::{Commands, Component, Res, Resource, Transform},
    sprite::{Sprite, SpriteBundle},
    window::Window,
};

use crate::settings::{TopLeftPosition, UiPanelSize};

#[derive(Debug, Clone)]
pub struct UiData {
    pub size: UiPanelSize,
    pub top_left_position: TopLeftPosition,
    pub color: Color,
}

#[derive(Debug, Component, Clone)]
pub struct UiPanel;

#[derive(Debug, Clone, Component)]
pub struct UiDataChanged;

#[derive(Debug, Resource, Clone)]
pub struct UI {
    size: UiPanelSize,
    top_left_position: TopLeftPosition,
    color: Color,
}

impl UI {
    pub fn new(ui_data: UiData) -> Self {
        Self {
            size: ui_data.size,
            top_left_position: ui_data.top_left_position,
            color: ui_data.color,
        }
    }

    pub fn update(&mut self, ui_data: UiData, window: &Window) {
        let ui_position = self.get_ui_window_xy(&ui_data.top_left_position, &ui_data.size, window);
        self.size = ui_data.size;
        self.top_left_position = ui_position;
        self.color = ui_data.color;
    }

    pub fn on_window_resize(&mut self, mut commands: Commands, window: &Window) {
        let ui_position = self.get_ui_window_xy(&self.top_left_position, &self.size, window);
        self.top_left_position = ui_position;
        println!("Window resized");
        commands.spawn(UiDataChanged);
    }

    pub fn to_transform(&self) -> Transform {
        // WILO: This seems to be producing outrageous values.
        let translation = Vec3::new(
            self.top_left_position.x as f32,
            self.top_left_position.y as f32,
            99.,
        );
        Transform::from_translation(translation)
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.size.width, self.size.height)
    }

    pub fn get_ui_bundle(&self) -> (SpriteBundle, UiPanel) {
        let bundle = (
            SpriteBundle {
                sprite: Sprite {
                    color: self.color,
                    custom_size: Some(self.to_vec2()),
                    ..Default::default()
                },
                transform: self.to_transform(),
                ..Default::default()
            },
            UiPanel,
        );

        println!("Bundle: {:#?}", bundle);

        bundle
    }

    // Privates
    fn get_ui_window_xy(
        &self,
        origin: &TopLeftPosition,
        box_size: &UiPanelSize,
        window: &Window,
    ) -> TopLeftPosition {
        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        let box_size_width = box_size.width * window.width();
        let box_size_height = box_size.height * window.height();
        let x = origin.x as f32;
        let y = origin.y as f32;

        TopLeftPosition {
            x: (x - half_width + box_size_width / 2.0) as usize,
            y: (y - half_height + box_size_height / 2.0) as usize,
        }
    }
}
