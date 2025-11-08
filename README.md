# Minecraft Note Block Music

Minecraft Note Block DAW entirely in Rust

## Usage
This uses a piano-roll layout, where left-right is time and up-down is pitch. Layers are on the bottom and instruments are per-layer, not per-note.

### Stuff:
- File menu: save your projects
- Edit menu: If you accidentally place note blocks above or below the piano and you can not reach them, you can delete them here
- Toolbar: Playback controlls and instruments
------------------------------------------------
- Scroll wheel: scroll left/right
- CLick left mouse button: Place or preview notes
- Drag left mouse button: select notes
- Middle mouse button: Move the music cursor thing around
- Right mouse button: Destroy notes
------------------------------------------------
- Ctrl+A: Select all notes
- Ctrl+D: Duplicate selection
- Delete: Delete selection
- Escape: Deselect
- Arrow keys: Move selection
------------------------------------------------
- Space: Play / pause
- Enter: stop

## Building
1. Clone repository
2. `cargo build` or `cargo run` in this folder
3. if you want to build it for release do `cargo build --release`
4. it should download all the dependencies and stuff for you because cargo is sigma