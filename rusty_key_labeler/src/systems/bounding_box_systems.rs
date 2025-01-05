use bevy::{math::VectorSpace, prelude::*};
use bevy_ui_views::VStackUpdatedItems;
use bevy_vector_shapes::{
    prelude::ShapeConfig,
    shapes::{DiscBundle, RectangleComponent, ShapeBundle, ShapeFill},
};
use itertools::Itertools;

use crate::{
    bounding_boxes::{BoundingBox, BoundingBoxPainter, ContainsBoundingBoxes, SelectedBoundingBox},
    resources::AppData,
    utils::{create_image_from_color, scale_dimensions},
    ImageReady, SelectedImage, Ui,
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
                let bounding_box = bb_painter.get_box(index, entry, image_size);
                let bounding_box_id = commands.spawn(bounding_box).id();
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

#[derive(Debug, Component, PartialEq)]
pub struct CornerHandle {
    x: f32,
    y: f32,
}

pub fn highlight_bounding_box(
    mut commands: Commands,
    mut selected_bounding_box: Query<
        (
            Entity,
            &BoundingBox,
            &mut ShapeFill,
            &mut RectangleComponent,
            &mut Transform,
        ),
        With<SelectedBoundingBox>,
    >,
    target_image: Query<
        &Sprite,
        (
            With<SelectedImage>,
            With<ImageReady>,
            With<ContainsBoundingBoxes>,
        ),
    >,
    images: ResMut<Assets<Image>>,
    bb_painter: Res<BoundingBoxPainter>,
) {
    if target_image.iter().count() == 0 {
        error!("No target image.");
        return;
    }

    let target_image = images.get(&target_image.single().image).unwrap();

    for (selected_bb_eid, bounding_box, mut shape_fill, mut rect, mut transform) in
        selected_bounding_box.iter_mut()
    {
        info!("Highlighting bounding box: {:?}", bounding_box.index);
        info!("Color: {:?}", shape_fill.color);
        info!("Rect: {:?}", rect.size);
        info!("Transform: {:?}", transform.translation);

        let (scaled_x_center, scaled_y_center, scaled_width, scaled_height) = scale_dimensions(
            transform.translation.x,
            transform.translation.y,
            rect.size.x,
            rect.size.y,
            Vec2::new(target_image.width() as f32, target_image.height() as f32),
        );

        let handle_size = 20.0;
        let half_handle_size = handle_size / 2.0;
        let half_width = scaled_width / 2.0;
        let half_height = scaled_height / 2.0;

        let top_left = Vec2::new(
            scaled_x_center - half_width + half_handle_size,
            scaled_y_center - half_height + half_handle_size,
        );

        let top_right = Vec2::new(scaled_x_center + half_width, scaled_y_center - half_height);

        let bottom_left = Vec2::new(scaled_x_center - half_width, scaled_y_center + half_height);

        let bottom_right = Vec2::new(scaled_x_center + half_width, scaled_y_center + half_height);

        let handles = [top_left, top_right, bottom_right, bottom_left];

        info!("{:#?}", handles);

        commands.entity(selected_bb_eid).despawn_descendants();

        for (index, handle) in handles.iter().enumerate() {
            let handle_component = (
                Name::new(format!("handle_{}", index)),
                ShapeBundle::circle(
                    &ShapeConfig {
                        color: bounding_box.class_color,
                        transform: Transform::from_translation(handle.extend(0.0)),
                        hollow: true,
                        thickness: bb_painter.bounding_box_settings.thickness,
                        corner_radii: Vec4::splat(bb_painter.bounding_box_settings.corner_radius),
                        ..ShapeConfig::default_2d()
                    },
                    handle_size,
                ),
            );

            let handle_component_id = commands.spawn(handle_component).id();
            commands
                .entity(selected_bb_eid)
                .add_child(handle_component_id);
        }

        // let box_handles = CornerHandles {
        //     top_left,
        //     top_right,
        //     bottom_left,
        //     bottom_right,
        // };

        // let handles = commands.spawn(box_handles).id();
        // commands.entity(bb_eid).add_child(handles);

        /////////////////////////////////////////////
        // mut alpha_descending: Local<bool>,
        // time: Res<Time>,
        // Throbbing color system
        // let mut alpha = shape_fill.color.alpha();
        // if alpha > 0.9 {
        //     *alpha_descending = true;
        // } else if alpha < 0.25 {
        //     *alpha_descending = false;
        // }

        // if *alpha_descending {
        //     alpha -= 0.75 * time.delta_secs();
        // } else {
        //     alpha += 0.75 * time.delta_secs();
        // }
        // shape_fill.color.set_alpha(alpha);
        /////////////////////////////////////////////
    }
}
