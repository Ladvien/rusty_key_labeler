use bevy::prelude::*;
use bevy_ui_views::VStackUpdatedItems;
use itertools::Itertools;

use crate::{
    bounding_boxes::{BoundingBoxPainter, ContainsBoundingBoxes},
    resources::AppData,
    utils::create_image_from_color,
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

    let (selected_image_eid, sprite) = match query.iter().next() {
        Some((eid, sprite)) => (eid, sprite),
        None => {
            error!("Selected image not found");
            return;
        }
    };

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
                let child_id = commands.spawn(bounding_box).id();
                children.push(child_id);

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
