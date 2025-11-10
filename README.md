# Minecraft Note Block Music

Minecraft Note Block DAW entirely in Rust

## Usage
This uses a piano-roll layout, where left-right is time and up-down is pitch.  
Layers are on the bottom and instruments are per-layer, not per-note.  
Notes that are greyed out are outside of Minecraft's note block range, so you won't be able to export it to Minecraft.

### Controlls:
- File menu: Save your projects
- Edit menu: Undo / Redo (does nothing)
- Toolbar: Playback controlls and instruments
------------------------------------------------
- Scroll wheel: Scroll around, the axises are swapped by default for convenience on mice so you can use shift to unswap them.
- Click left mouse button: Place or hear notes
- Drag left mouse button: Select notes
- Middle mouse button: Move the playback line around
- Right mouse button: Destroy notes
- R: Reset scroll
------------------------------------------------
- Ctrl+A: Select all notes
- Ctrl+D: Duplicate selection
- Delete: Delete selection
- Escape: Deselect
- Arrow keys: Move selection
------------------------------------------------
- Space: Play / pause
- Enter: Stop

## Building
1. Clone repository
2. In this folder run `cargo build` to just build it or `cargo run` to build it and run it. If you want to make a release do `cargo build --release` because you probably don't need debug info in the release.