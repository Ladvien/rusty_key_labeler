use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct ImageLoading(pub Handle<Image>);

#[derive(Debug, Clone, Component)]
pub struct ImageReady(pub Handle<Image>);

#[derive(Debug, Clone, Component)]
pub struct SelectedImage;

#[derive(Debug, Clone, Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Component)]
pub struct DebounceTimer {
    pub timer: Timer,
}

#[derive(Debug, Clone, Component)]
pub struct FocusViewport {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default, Component)]
pub struct CanvasMarker;

#[derive(Debug, Clone, Default, Component)]
pub struct ComputedViewport {
    pub width: f32,
    pub height: f32,
    pub translation: Vec3,
}

#[derive(Debug, Clone, Default, Component)]
pub struct UninitializedRenderTarget;
