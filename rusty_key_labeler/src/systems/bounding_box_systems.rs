use bevy::{
    color::palettes::tailwind::{RED_50, RED_700},
    prelude::*,
};
use bevy_ui_views::VStackUpdatedItems;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{DiscBundle, RectangleBundle, RectangleComponent, ShapeBundle, ShapeFill},
};
use itertools::Itertools;

use crate::{
    bounding_boxes::{BoundingBox, BoundingBoxPainter, ContainsBoundingBoxes, SelectedBoundingBox},
    resources::AppData,
    utils::create_image_from_color,
    CenterInViewport, FocusInViewport, ImageReady, MainCamera, SelectedImage, Ui,
};

pub fn load_bounding_boxes(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    query: Query<
        (Entity, &Sprite),
        (
            With<SelectedImage>,
            With<ImageReady>,
            Without<ContainsBoundingBoxes>,
        ),
    >,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
    ui: Res<Ui>,
) {
    if query.iter().count() == 0 {
        return;
    }

    debug!("Painting bounding boxes");

    let pair = match app_data.yolo_project.pair_at_index(app_data.index) {
        Some(pair) => pair,
        None => {
            error!("Pair not found");
            return;
        }
    };

    let yolo_file = match pair.label_file {
        Some(file) => file,
        None => {
            error!("Label file not found");
            return;
        }
    };

    let mut children = Vec::new();
    let mut ui_items = Vec::new();

    let (selected_image_eid, sprite) = query.single();

    debug!("Selected image: {:?}", sprite.image.id());

    match images.get_mut(&sprite.image) {
        Some(image) => {
            // TODO: Keep an eye on this.
            // TODO: What happens if this fails continually?
            commands
                .entity(selected_image_eid)
                .try_insert(ContainsBoundingBoxes);

            let image_size = Vec2::new(image.width() as f32, image.height() as f32);

            for (index, entry) in yolo_file
                .entries
                .iter()
                .enumerate()
                .sorted_by_key(|(_, entry)| {
                    // Sort by area. This allows for consistent top-right
                    // to bottom-left ordering.
                    (entry.x_center * 1000.0) as u32 + (entry.y_center * 1000.0) as u32
                })
                .rev()
            {
                //
                debug!("Adding bounding box: {}", index);
                let bounding_box_id =
                    bb_painter.spawn_bounding_box(&mut commands, index, entry, image_size);

                children.push(bounding_box_id);

                let color = bb_painter.get_color(entry.class);

                // TODO: I should preload all the color swatches, giving them a path.
                let image = create_image_from_color(color, 40, 40);
                let image_handle = images.add(image);

                let item = ui.create_bounding_box_entry(
                    &app_data.yolo_project.config.export.class_map[&entry.class],
                    image_handle,
                );

                ui_items.push(item);
            }

            // Add bounding box references to UI
            if let Some(left_panel_eid) = app_data.left_panel_eid {
                debug!("Updating left panel");
                commands.spawn(VStackUpdatedItems {
                    items: ui_items.clone(),
                    vstack_eid: left_panel_eid,
                });
            }
        }
        None => {
            error!("Image not found");
            return;
        }
    };
    if !children.is_empty() {
        debug!("Adding children to selected image");
        commands.entity(selected_image_eid).add_children(&children);
    }
}

#[derive(Debug, Component, PartialEq)]
pub struct CornerHandles {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_left: Vec2,
    pub bottom_right: Vec2,
}

#[derive(Debug, Clone, Component, PartialEq)]
pub struct CornerHandle {
    pub name: String,
    pub position: Vec2,
}

pub fn highlight_bounding_box(
    mut commands: Commands,
    corner_handles: Query<Entity, With<CornerHandle>>,
    mut selected_bounding_box: Query<
        (Entity, &BoundingBox, &mut RectangleComponent),
        Changed<SelectedBoundingBox>,
    >,
    bb_painter: Res<BoundingBoxPainter>,
    app_data: Res<AppData>,
) {
    if selected_bounding_box.iter().count() == 0 {
        return;
    }

    for corner_handle_eid in corner_handles.iter() {
        commands.entity(corner_handle_eid).despawn_recursive();
    }

    for (selected_bb_eid, bounding_box, rect) in selected_bounding_box.iter_mut() {
        let handle_size = app_data.config.settings.bounding_boxes.handle_size;

        // Offset handles.
        let top_left = CornerHandle {
            name: String::from("top_left"),
            position: Vec2::new(-1.0 * (rect.size.x / 2.0), rect.size.y / 2.0),
        };

        let top_right = CornerHandle {
            name: String::from("top_right"),
            position: Vec2::new(rect.size.x / 2.0, rect.size.y / 2.0),
        };

        let bottom_left = CornerHandle {
            name: String::from("bottom_left"),
            position: Vec2::new(-1.0 * (rect.size.x / 2.0), -1.0 * (rect.size.y / 2.0)),
        };

        let bottom_right = CornerHandle {
            name: String::from("bottom_right"),
            position: Vec2::new(rect.size.x / 2.0, -1.0 * (rect.size.y / 2.0)),
        };

        let handles = [top_left, top_right, bottom_left, bottom_right];

        commands.entity(selected_bb_eid).despawn_descendants();

        for handle in handles.iter() {
            let handle_component = (
                Name::new(handle.name.clone()),
                ShapeBundle::rect(
                    &ShapeConfig {
                        color: Color::from(RED_700),
                        transform: Transform::from_translation(handle.position.extend(999.0)),
                        hollow: true,
                        thickness: bb_painter.bounding_box_settings.thickness,
                        corner_radii: Vec4::splat(bb_painter.bounding_box_settings.corner_radius),
                        ..ShapeConfig::default_2d()
                    },
                    Vec2::splat(handle_size),
                ),
                handle.clone(),
            );

            let handle_component_id = commands.spawn(handle_component).id();
            commands
                .entity(selected_bb_eid)
                .add_child(handle_component_id);
        }
    }
}
