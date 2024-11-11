# rusty_key_labeler
A Rust YOLO Labeler with a focus on pure keyboard

## Features
- Keyboard first image labeler
- Customizable shortcuts
- Tab based indexing of bounding-boxes

## TODO
- [x] Circular rotation of image index
- [X] Add UI
  - [ ] Rebuild UI panel using [Bevy Lunex](https://github.com/bytestring-net/bevy_lunex/tree/main)
  - [ ] Show "File x / n"
  - [ ] Show current image name / path
  - [ ] Show current label name / path
  - [ ] Show the name of the classes and a color swatch
- [ ] Bubble up errors to UI
- [ ] Highlight selected bounding box
- [ ] Cmd+Z
- [X] HUD
  - [ ] Name of current image, path to label, display class map
- [X] Adjustable bounding box thickness, color, etc.
  - [X] If no `class_color_map` is provided, then classic colors are mapped at startup.
- [ ] Bounding boxes automatically adjusted if hard to see
- [ ] Image automatically fits to screen on load