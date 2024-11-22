use bevy::{
    math::VectorSpace,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    sprite::Anchor,
};

use bevy_lunex::prelude::*;

use crate::{
    settings::{TopLeftPosition, UiPanelSettings, UiPanelSize, UI_LAYER},
    ImageData, TestFlag, UiData,
};

pub const UI_Z_INDEX: f32 = 99.0;

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
    old_ui_eid: Option<Entity>,
}

impl UI {
    pub fn new(ui_data: UiPanelSettings) -> Self {
        Self {
            size: ui_data.size,
            original_top_left_position: ui_data.top_left_position.clone(),
            top_left_position: ui_data.top_left_position,
            color: ui_data.color,
            old_ui_eid: None,
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

    pub fn to_transform(&self, window: &Window) -> Transform {
        let half_width = window.width() / 2.;
        let half_height = window.height() / 2.;
        let half_box_width = self.size.width / 2.;
        let half_box_height = self.size.height / 2.;
        let x = self.top_left_position.x as f32 - half_width + half_box_width;
        let y = self.top_left_position.y as f32 - half_height + half_box_height;

        let translation = Vec3::new(x, y, UI_Z_INDEX);

        Transform::from_translation(translation)
    }

    pub fn paint_ui(
        &mut self,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        window: &Window,
        data: Option<&ImageData>,
    ) {
        println!("{:#?}", data);

        let x = self.top_left_position.x as f32;
        let y = self.top_left_position.y as f32;

        let bottom_left_position = Ab((
            window.width() - self.size.width,
            window.height() - self.size.height,
        ));

        let font_handle = asset_server.load("RobotoMono-Regular.ttf");

        // Unwrap UI data.
        let stem = data.map(|d| d.stem.clone()).unwrap_or(String::from(""));

        // Spawn UiTree
        let root = UiTreeBundle::<MainUi> {
            // transform,
            tree: UiTree::new2d("Root"),
            ..default()
        };

        let new_ui_eid = commands
            .spawn((
                root.clone(),
                Name::new("root"),
                SourceFromCamera,
                UiPanel,
                UI_LAYER,
            ))
            .with_children(|ui| {
                ui.spawn((
                    // Link the entity
                    UiLink::<MainUi>::path("root"),
                    // Specify UI layout
                    UiLayout::window()
                        .pos(bottom_left_position)
                        .size(Ab(Vec2::new(self.size.width, self.size.height)))
                        .anchor(Anchor::TopLeft)
                        .pack::<Base>(),
                ));

                ui.spawn((
                    // Link the entity
                    UiLink::<MainUi>::path("root/rectangle"),
                    // Specify UI layout
                    UiLayout::solid()
                        .size(Ab(Vec2::new(self.size.width, self.size.height)))
                        .pack::<Base>(),
                    // Add image to the entity
                    UiImage2dBundle {
                        sprite: Sprite {
                            color: self.color,
                            custom_size: Some(Vec2::new(self.size.width, self.size.height)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(Vec3::new(x, y, UI_Z_INDEX)),
                        ..Default::default()
                    },
                    // UiImage2dBundle::from(asset_server.load("background.png")),
                    TestFlag,
                    UiPanel,
                    UI_LAYER,
                ));

                // ui.spawn((
                //     // Link this widget
                //     UiLink::<MainUi>::path("root/rectangle/label"),
                //     // Here we can define where we want to position our text within the parent node,
                //     // don't worry about size, that is picked up and overwritten automaticaly by Lunex to match text size.
                //     UiLayout::window()
                //         .pos(main_label_spacing)
                //         .anchor(Anchor::TopLeft)
                //         .pack::<Base>(),
                //     // Add text
                //     UiText2dBundle {
                //         text: Text::from_section(
                //             "Info:",
                //             TextStyle {
                //                 font: font_handle.clone(),
                //                 font_size: 12.0,
                //                 color: Color::WHITE,
                //             },
                //         ),
                //         ..default()
                //     },
                //     UiPanel,
                //     UI_LAYER,
                // ));

                ui.spawn((
                    UiLink::<MainUi>::path("root/rectangle/image_index"),
                    UiLayout::window()
                        .pos(Ab((5.0, 5.0)))
                        .anchor(Anchor::TopLeft)
                        .pack::<Base>(),
                    UiText2dBundle {
                        text: Text::from_section(
                            format!(
                                "Image Index: {} of {}",
                                data.map(|d| d.index).unwrap_or(0),
                                data.map(|d| d.total_images).unwrap_or(0)
                            ),
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 12.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    },
                    UiPanel,
                    UI_LAYER,
                ));

                ui.spawn((
                    UiLink::<MainUi>::path("root/rectangle/image_name"),
                    UiLayout::window()
                        .anchor(Anchor::TopLeft)
                        .pos(Ab((5.0, 20.0)))
                        .pack::<Base>(),
                    UiText2dBundle {
                        text: Text::from_section(
                            format!("Image Name: {}", stem),
                            TextStyle {
                                font: font_handle.clone(),
                                font_size: 12.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    },
                    UiPanel,
                    UI_LAYER,
                ));
            })
            .id();

        if let Some(old_ui_eid) = self.old_ui_eid {
            commands.entity(old_ui_eid).despawn_recursive();
        }

        self.old_ui_eid = Some(new_ui_eid);
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

fn color_to_float_array(color: Color) -> [f32; 4] {
    let r = (color.to_linear().red) as f32;
    let g = (color.to_linear().green) as f32;
    let b = (color.to_linear().blue) as f32;
    let a = (color.to_linear().alpha) as f32;

    [r, g, b, a]
}

pub fn create_image_from_color(images: &mut ResMut<Assets<Image>>, color: Color) -> Handle<Image> {
    let color_data = color_to_float_array(color);
    let pixel_data = color_data
        .into_iter()
        .flat_map(|channel| channel.to_ne_bytes())
        .collect::<Vec<_>>();

    // println!("Pixel data: {:#?}", pixel_data);

    let image = Image::new_fill(
        Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &pixel_data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD,
    );

    let texture_handle = images.add(image);

    texture_handle
}
