use bevy::{math::VectorSpace, prelude::*, sprite::Anchor};

use bevy_lunex::prelude::*;

use crate::settings::{TopLeftPosition, UiPanelSettings, UiPanelSize, UI_LAYER};

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
        &self,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        window: &Window,
    ) {
        // let transform = self.to_transform(window);

        let half_window_width = window.width() / 2.;
        let half_window_height = window.height() / 2.;
        let half_box_width = self.size.width / 2.;
        let half_box_height = self.size.height / 2.;
        let x = self.top_left_position.x as f32;
        let y = self.top_left_position.y as f32;

        let bottom_left_position = Ab((
            window.width() - self.size.width,
            window.height() - self.size.height,
        ));

        let spacing = Rl((5., 5.));
        let top_padding = Rl(2.0);
        let font_handle = asset_server.load("RobotoMono-Regular.ttf");

        // Spawn UiTree
        commands
            .spawn((
                UiTreeBundle::<MainUi> {
                    // transform,
                    tree: UiTree::new2d("root"),
                    ..default()
                },
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
                    UiPanel,
                    UI_LAYER,
                ));

                ui.spawn((
                    // Link this widget
                    UiLink::<MainUi>::path("root/rectangle/text"),
                    UiLayout::boundary()
                        .pos1(top_padding)
                        .pos2(Rl(80.0))
                        .pack::<Base>(),
                    // Add text
                    UiText2dBundle {
                        text: Text::from_section(
                            "Info:",
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

                // ui.spawn((
                //     // Link this widget
                //     UiLink::<MainUi>::path("root/rectangle/text2"),
                //     UiLayout::boundary()
                //         .pos1(top_padding)
                //         // .pos2(Rl(80.0))
                //         .pack::<Base>(),
                //     // Add text
                //     UiText2dBundle {
                //         text: Text::from_section(
                //             "Hello World!",
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

                // Spawn image with a node
                // ui.spawn((
                //     UiLink::<MainUi>::path("root/image"),
                //     // UiLayout::solid()
                //     //     .size((Ab(self.size.width), Ab(self.size.height)))
                //     //     .pack::<Base>(),
                //     UiImage2dBundle::from(asset_server.load("background.png")),
                //     UiPanel,
                //     UI_LAYER,
                // ));
            });

        // let bundle = (
        //     Name::new("ui"),
        //     SpriteBundle {
        //         sprite: Sprite {
        //             color: self.color,
        //             custom_size: Some(Vec2::new(self.size.width, self.size.height)),
        //             ..Default::default()
        //         },
        //         transform,
        //         ..Default::default()
        //     },
        //     UiPanel,
        //     UI_LAYER,
        // );

        // let ui_eid = commands.spawn(bundle).id();

        // println!("UI Entity ID: {:#?}", ui_eid);

        // commands.spawn((
        //     // #=== UI DEFINITION ===#

        //     // This specifies the name and hierarchy of the node
        //     UiLink::<MainUi>::path("Menu/Button"),
        //     // Here you can define the layout using the provided units (per state like Base, Hover, Selected, etc.)
        //     UiLayout::window()
        //         .pos(Rl((
        //             self.top_left_position.x as f32,
        //             self.top_left_position.y as f32,
        //         )))
        //         .size(Rh(Vec2::new(self.size.width, self.size.height)))
        //         .pack::<Base>(),
        //     // #=== CUSTOMIZATION ===#

        //     // Give it a background image
        //     UiImage2dBundle {
        //         sprite: Sprite {
        //             color: self.color,
        //             custom_size: Some(Vec2::new(self.size.width, self.size.height)),
        //             ..Default::default()
        //         },
        //         transform,
        //         ..Default::default()
        //     },
        //     // // Make the background image resizable
        //     // ImageScaleMode::Sliced(TextureSlicer { border: BorderRect::square(32.0), ..default() }),

        //     // // This is required to control our hover animation
        //     // UiAnimator::<Hover>::new().forward_speed(5.0).backward_speed(1.0),

        //     // // This will set the base color to red
        //     UiColor<Base>::new(Color::RED),

        //     // // This will set hover color to yellow
        //     // UiColor::<UiColor>::new(Color::YELLOW),
        //     // // #=== INTERACTIVITY ===#

        //     // // This is required for hit detection (make it clickable)
        //     // PickableBundle::default(),

        //     // // This will change cursor icon on mouse hover
        //     // OnHoverSetCursor::new(CursorIcon::Pointer),

        //     // // If we click on this, it will emmit UiClick event we can listen to
        //     // UiClickEmitter::SELF,
        //     UiPanel,
        //     UI_LAYER,
        // ));
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
