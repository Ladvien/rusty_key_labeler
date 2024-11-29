use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_ui_views::VStack;

use crate::settings::{UiColors, UI_LAYER};

pub const UI_Z_INDEX: f32 = 99.0;
pub const PADDING: f32 = 5.0;

#[derive(Debug, Clone, Component)]
pub struct UiCamera;

#[derive(Debug, Clone, Component)]
pub struct UiData {
    pub stem: String,
    pub image_path: String,
    pub label_path: String,
}

#[derive(Debug, Clone, Component)]
pub struct UILeftPanel;

#[derive(Debug, Clone, Component)]
pub struct UIBottomPanel;

#[derive(Debug, Component, Clone)]
pub struct UiPanel;

#[derive(Debug, Clone, Component)]
pub struct UiDataChanged;

#[derive(Debug, Clone, Resource)]
pub struct Ui {
    pub colors: UiColors,
    pub font_size: f32,
    pub font: Handle<Font>,
}

impl Ui {
    pub fn new(colors: &UiColors) -> Self {
        Self {
            colors: colors.clone(),
            font_size: 20.0,
            font: Default::default(),
        }
    }

    pub fn spawn_ui(&self, commands: &mut Commands) -> Entity {
        // Spawn the UI Container
        let ui_eid = commands
            .spawn((
                Name::new("UIContainer"),
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, UI_Z_INDEX),
                        ..default()
                    },
                    ..default()
                },
                UILeftPanel,
                UI_LAYER,
            ))
            .id();

        let vstack_eid = commands
            .spawn((
                Name::new("VStack"),
                VStack {
                    text: "ExtendedScrollView".to_string(),
                    position: Vec2::new(0.0, 0.0),
                    percent_width: 25.0,
                    percent_height: 90.0,
                    layer: UI_LAYER,
                    background_color: self.colors.background,
                    border_color: self.colors.outer_border,
                    border: UiRect {
                        top: Val::Px(1.0),
                        left: Val::Px(1.0),
                        right: Val::Px(1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .id();

        commands.entity(ui_eid).push_children(&[vstack_eid]);

        let bottom_ui_eid = commands
            .spawn((
                Name::new("bottom_ui_panel"),
                NodeBundle {
                    // Here is where all the styling goes for the container, duh.
                    style: Style {
                        width: Val::Percent(100.0),
                        min_height: Val::Percent(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect {
                            left: Val::Px(PADDING),
                            right: Val::Px(PADDING),
                            top: Val::Px(PADDING),
                            bottom: Val::Px(PADDING),
                        },
                        ..default()
                    },
                    border_color: self.colors.outer_border.into(),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, UI_Z_INDEX),
                        ..default()
                    },
                    background_color: BackgroundColor(self.colors.background),
                    ..default()
                },
                UIBottomPanel,
                UI_LAYER,
            ))
            .with_children(|parent| {
                parent.spawn((TextBundle {
                    style: Style {
                        min_height: Val::Px(20.0),
                        min_width: Val::Px(100.0),
                        ..default()
                    },
                    text: Text {
                        sections: vec![TextSection {
                            value: String::from("0/0"),
                            style: TextStyle {
                                font_size: self.font_size,
                                color: self.colors.text,
                                ..default()
                            },
                        }],
                        ..Default::default()
                    },
                    // background_color: BackgroundColor::from(),
                    ..Default::default()
                },));
            })
            .id();

        commands.entity(ui_eid).push_children(&[bottom_ui_eid]);

        ui_eid
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
