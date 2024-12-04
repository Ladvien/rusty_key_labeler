use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_ui_views::{VStack, VStackContainerItem};

use crate::{
    settings::{UiColors, UI_LAYER},
    AppData,
};

pub const UI_Z_INDEX: f32 = 99.0;
pub const PADDING: f32 = 5.0;

#[derive(Debug, Clone, Component)]
pub struct UiCamera;

#[derive(Debug, Clone, Component)]
pub struct UILeftPanel;

#[derive(Debug, Clone, Component)]
pub struct UIBottomPanel;

#[derive(Debug, Clone, Component)]
pub struct UiLabelDataChanged;

// UI Part Markers
#[derive(Debug, Clone, Component)]
pub struct UiLabelingIndex;

#[derive(Debug, Clone, Component)]
pub struct UiLabelingIndexUpdateNeeded(pub String);

#[derive(Debug, Clone, Component)]
pub struct CurrentFileNameLabel;

#[derive(Debug, Clone, Component)]
pub struct CurrentFileNameLabelUpdateNeeded(pub String);

// END UI Part Markers

#[derive(Debug, Clone, Resource)]
pub struct Ui {
    pub colors: UiColors,
    pub font_size: f32,
    pub font_path: String,
    font_handle: Option<Handle<Font>>,
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
        Camera2dBundle {
            camera: Camera {
                // Render the UI on top of everything else.
                order: 1,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        UI_LAYER,
        UiCamera,
    ));

    let ui_eid = ui.spawn_ui(&mut commands);

    app_data.ui_eid = Some(ui_eid);
}

// Systems on Update
pub fn update_labeling_index(
    mut commands: Commands,
    mut query: Query<&mut Text, With<UiLabelingIndex>>,
    update_query: Query<(Entity, &UiLabelingIndexUpdateNeeded)>,
) {
    for (update_eid, update) in update_query.iter() {
        for mut text in query.iter_mut() {
            text.sections[0].value = update.0.clone();
            commands.entity(update_eid).despawn();
        }
    }
}

pub fn update_current_file_name_label(
    mut commands: Commands,
    mut query: Query<&mut Text, With<CurrentFileNameLabel>>,
    update_query: Query<(Entity, &CurrentFileNameLabelUpdateNeeded)>,
) {
    for (update_eid, update) in update_query.iter() {
        for mut text in query.iter_mut() {
            text.sections[0].value = update.0.clone();
            commands.entity(update_eid).despawn();
        }
    }
}

impl Ui {
    pub fn new(colors: &UiColors, font_size: f32, font_path: &str) -> Self {
        Self {
            colors: colors.clone(),
            font_size,
            font_path: font_path.to_string(),
            font_handle: None,
        }
    }

    pub fn spawn_ui(&self, commands: &mut Commands) -> Entity {
        // Spawn the UI Container
        let ui_eid = commands
            .spawn((
                Name::new("left_ui_panel"),
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
            .with_children(|bottom_ui_panel| {
                bottom_ui_panel.spawn((
                    Name::new("labeling_index"),
                    TextBundle {
                        style: Style {
                            min_height: Val::Px(20.0),
                            min_width: Val::Px(100.0),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection {
                                value: String::from("0/0"),
                                style: TextStyle {
                                    font: self.font_handle.clone().unwrap(),
                                    font_size: self.font_size,
                                    color: self.colors.text,
                                },
                            }],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    UiLabelingIndex,
                ));

                bottom_ui_panel.spawn((
                    Name::new("current_file_name"),
                    TextBundle {
                        style: Style {
                            min_height: Val::Px(20.0),
                            min_width: Val::Px(100.0),
                            ..default()
                        },
                        text: Text {
                            sections: vec![TextSection {
                                value: String::from(""),
                                style: TextStyle {
                                    font: self.font_handle.clone().unwrap(),
                                    font_size: self.font_size,
                                    color: self.colors.text,
                                },
                            }],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    CurrentFileNameLabel,
                ));
            })
            .id();

        commands.entity(ui_eid).push_children(&[bottom_ui_eid]);

        ui_eid
    }

    pub fn create_bounding_box_entry(
        &self,
        text: &str,
        class_image: Handle<Image>,
    ) -> VStackContainerItem {
        VStackContainerItem {
            text: text.to_string(),
            image: Some(class_image),
            text_color: self.colors.text,
            border_color: self.colors.outer_border,
            ..Default::default()
        }
    }

    pub fn create_image_from_color(&self, color: Color) -> Image {
        let color_data = color_to_float_array(color);
        let pixel_data = color_data
            .into_iter()
            .flat_map(|channel| channel.to_ne_bytes())
            .collect::<Vec<_>>();

        // println!("Pixel data: {:#?}", pixel_data);

        Image::new_fill(
            Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &pixel_data,
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

fn color_to_float_array(color: Color) -> [f32; 4] {
    let r = color.to_linear().red;
    let g = color.to_linear().green;
    let b = color.to_linear().blue;
    let a = color.to_linear().alpha;

    [r, g, b, a]
}
