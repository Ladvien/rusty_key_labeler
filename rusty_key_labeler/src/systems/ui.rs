use crate::utils::create_image_from_color;
use crate::{
    settings::UiColors, AppData, CurrentFileNameLabel, CurrentFileNameLabelUpdateNeeded,
    UIBottomPanel, UILeftPanel, UITopPanel, UiBasePanel, UiLabelingIndex,
    UiLabelingIndexUpdateNeeded,
};
use crate::{ImageViewport, MainCamera, Ui};
use bevy::render::view::RenderLayers;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::{color::palettes::css::*, prelude::*};
use bevy_ui_views::{VStack, VStackContainerItem};

use super::VIEWPORT_LAYER;

pub const CANVAS_Z_INDEX: i32 = 0;
pub const UI_Z_INDEX: f32 = 99.0;
pub const PADDING: f32 = 5.0;

// Systems on Setup
// pub fn ui_setup(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut app_data: ResMut<AppData>,
//     mut ui: ResMut<Ui>,
//     mut images: ResMut<Assets<Image>>,
// ) {

// }

// Systems on Update
pub fn update_labeling_index(
    mut commands: Commands,
    mut query: Query<&mut Text, With<UiLabelingIndex>>,
    update_query: Query<(Entity, &UiLabelingIndexUpdateNeeded)>,
) {
    for (update_eid, update) in update_query.iter() {
        for mut text in query.iter_mut() {
            text.0 = update.0.clone();
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
            text.0 = update.0.clone();
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

    pub fn spawn_ui(
        &self,
        commands: &mut Commands,
        viewport_image: Handle<Image>,
    ) -> (Entity, Entity) {
        // Spawn the UI Container
        let container_eid = commands
            .spawn((
                Name::new("ui_container"),
                UiBasePanel,
                Node {
                    flex_direction: FlexDirection::Column,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(4.0)),
                    overflow: Overflow {
                        x: OverflowAxis::Clip,
                        y: OverflowAxis::Clip,
                    },
                    ..Default::default()
                },
                BorderColor(self.colors.outer_border),
                // BorderColor(Color::from(ORANGE)),
                ZIndex(UI_Z_INDEX as i32),
            ))
            .id();

        let top_half_panel = commands
            .spawn((
                Name::new("top_half_panel"),
                Node {
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    // border: UiRect::all(Val::Px(1.0)),
                    // padding: UiRect {
                    //     left: Val::Px(0.0),
                    //     right: Val::Px(PADDING),
                    //     top: Val::Px(0.0),
                    //     bottom: Val::Px(PADDING),
                    // },
                    ..default()
                },
                BorderColor(self.colors.outer_border),
                // BackgroundColor(self.colors.background),
                Transform {
                    translation: Vec3::new(0.0, 0.0, UI_Z_INDEX),
                    ..default()
                },
                UITopPanel,
                ZIndex(UI_Z_INDEX as i32),
            ))
            .id();

        commands.entity(container_eid).add_child(top_half_panel);

        let left_panel_ui_eid = commands
            .spawn((
                Name::new("left_ui_panel"),
                Node {
                    // flex_direction: FlexDirection::Column,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    min_width: Val::Percent(20.0),
                    max_width: Val::Percent(20.0),
                    width: Val::Percent(20.0),
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::new(0.0, 0.0, UI_Z_INDEX),
                    ..default()
                },
                BackgroundColor(self.colors.background),
                // BackgroundColor(Color::from(ORANGE_RED)),
                UILeftPanel,
                ZIndex(UI_Z_INDEX as i32),
            ))
            .id();

        commands.entity(top_half_panel).add_child(left_panel_ui_eid);

        let right_top_panel_ui_eid = commands
            .spawn((
                Name::new("right_top_panel"),
                Node {
                    // position_type: PositionType::Absolute,
                    overflow: Overflow {
                        x: OverflowAxis::Hidden,
                        y: OverflowAxis::Hidden,
                    },
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    // width: Val::Percent(80.0),
                    // max_width: Val::Percent(80.0),
                    // min_width: Val::Percent(80.0),
                    // height: Val::Percent(100.0),
                    // min_height: Val::Percent(100.0),
                    // max_height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode {
                    image: viewport_image.clone_weak(),
                    ..Default::default()
                },
                ImageViewport,
                ZIndex(CANVAS_Z_INDEX),
                VIEWPORT_LAYER,
            ))
            .id();

        commands
            .entity(top_half_panel)
            .add_child(right_top_panel_ui_eid);

        let vstack_eid = commands
            .spawn((
                Name::new("VStack"),
                VStack {
                    text: "ExtendedScrollView".to_string(),
                    position: Vec2::new(0.0, 0.0),
                    percent_width: 100.0,
                    percent_height: 100.0,
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
                ZIndex(UI_Z_INDEX as i32),
            ))
            .id();

        commands.entity(left_panel_ui_eid).add_child(vstack_eid);

        let bottom_ui_eid = commands
            .spawn((
                UIBottomPanel,
                Name::new("bottom_ui_panel"),
                Node {
                    flex_direction: FlexDirection::Column,
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
                BorderColor(self.colors.outer_border),
                BackgroundColor(self.colors.background),
                Transform {
                    translation: Vec3::new(0.0, 0.0, UI_Z_INDEX),
                    ..default()
                },
                ZIndex(UI_Z_INDEX as i32),
            ))
            .insert(PickingBehavior {
                should_block_lower: false,
                ..default()
            })
            .with_children(|bottom_ui_panel| {
                bottom_ui_panel.spawn((
                    Name::new("labeling_index"),
                    Text::from("0/0"),
                    TextFont {
                        font: self.font_handle.clone().unwrap(),
                        font_size: self.font_size,
                        ..Default::default()
                    },
                    TextColor::from(self.colors.text),
                    UiLabelingIndex,
                ));

                bottom_ui_panel.spawn((
                    Name::new("current_file_name"),
                    Text::from(""),
                    TextFont {
                        font: self.font_handle.clone().unwrap(),
                        font_size: self.font_size,
                        ..Default::default()
                    },
                    TextColor::from(self.colors.text),
                    Node {
                        min_height: Val::Px(20.0),
                        min_width: Val::Px(100.0),
                        ..Default::default()
                    },
                    CurrentFileNameLabel,
                ));
            })
            .id();

        commands.entity(container_eid).add_child(bottom_ui_eid);

        // Return the container entity ID
        (container_eid, left_panel_ui_eid)
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
}
