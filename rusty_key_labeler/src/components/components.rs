use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct ImageLoading(pub Handle<Image>);

#[derive(Debug, Clone, Component)]
pub struct ImageReady(pub Handle<Image>);

#[derive(Debug, Clone, Component, Default)]
pub struct SelectedImage;

#[derive(Debug, Clone, Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Component)]
pub struct DebounceTimer {
    pub timer: Timer,
}

#[derive(Debug, Clone, Component)]
pub struct FocusInViewport {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Component)]
pub struct CenterInViewport;

#[derive(Debug, Clone, Default, Component)]
pub struct CanvasMarker;

#[derive(Debug, Clone, Default, Component, Reflect)]
pub struct ComputedViewport {
    pub width: f32,
    pub height: f32,
    pub translation: Vec3,
}

#[derive(Debug, Clone, Default, Component)]
pub struct UninitializedRenderTarget;
