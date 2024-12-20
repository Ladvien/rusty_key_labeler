use bevy::prelude::Component;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::{color::palettes::css::*, prelude::*};
use bevy_ui_views::{VStack, VStackContainerItem};

use crate::{
    settings::{UiColors, UI_LAYER},
    AppData,
};

#[derive(Debug, Clone, Component)]
pub struct TopRightPanelUI;

#[derive(Debug, Clone, Default, Component)]
pub struct CanvasPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Component)]
pub struct UiCamera;

#[derive(Debug, Clone, Component)]
pub struct UiBasePanel;

#[derive(Debug, Clone, Component)]
pub struct UILeftPanel;

#[derive(Debug, Clone, Component, Default)]
#[require(Name, Node, Transform, BorderColor, BackgroundColor)]
pub struct UIBottomPanel;

#[derive(Debug, Clone, Component)]
pub struct UITopPanel;

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
    pub font_handle: Option<Handle<Font>>,
}
