use bevy::prelude::Component;
use bevy::prelude::*;

use crate::settings::UiColors;

#[derive(Debug, Clone, Component)]
pub struct TopRightPanelUI;

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

// UI Part Markers
#[derive(Debug, Clone, Component)]
pub struct UiLabelingIndex;

#[derive(Debug, Clone, Component)]
pub struct UiLabelingIndexUpdateNeeded(pub String);

#[derive(Debug, Clone, Component)]
pub struct CurrentFileNameLabel;

#[derive(Debug, Clone, Component)]
pub struct FileNameLabelUpdateNeeded(pub String);

// END UI Part Markers

#[derive(Debug, Clone, Resource)]
pub struct Ui {
    pub colors: UiColors,
    pub font_size: f32,
    pub font_path: String,
    pub font_handle: Option<Handle<Font>>,
}
