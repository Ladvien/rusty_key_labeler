# rusty_key_labeler
A Rust YOLO Labeler with a focus on pure keyboard

## Features
- Keyboard first image labeler
- Customizable shortcuts
- Tab based indexing of bounding-boxes

## TODO
- [x] Circular rotation of image index
- [ ] Add UI
- [ ] Bubble up errors to UI
- [ ] Highlight selected bounding box
- [ ] Cmd+Z
- [ ] HUD
  - [ ] Name of current image, path to label, display class map
- [ ] Adjustable bounding box thickness, color, etc.
  - [ ] If no `class_color_map` is provided, then classic colors are mapped at startup.
- [ ] Bounding boxes automatically adjusted if hard to see
- [ ] Image automatically fits to screen on load