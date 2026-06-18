# iced_color_picker

An [Iced](https://iced.rs/) color picker: HSV hue/saturation disc, value bar, optional alpha bar, RGB(A) sliders, hex field, and copy control.

![Color picker demo](assets/screenshot.png)

**Iced 0.14** with the `canvas`, `svg`, and `advanced` features (enable the same features on `iced` in your app).

## Features

- Circular **hue / saturation** disc and vertical **value** bar
- Optional **alpha** bar (enable via `ColorPickerState::from_rgba`)
- **R**, **G**, **B** (and optional **A**) sliders — can be hidden with `.show_sliders(false)`
- **#RRGGBB** / **#RRGGBBAA** hex input
- **Copy** button (your app handles `PickerMessage::CopyHex` and writes the clipboard)

No preset palette or swatch grid; only the picker UI and `ColorPickerState`.

## Examples

```bash
cargo run --example rgb
cargo run --example rgba
```

## Usage

```rust
use iced_color_picker::{ColorPickerState, PickerMessage, color_picker};
use iced_color_picker::{PICKER_PANEL_WIDTH, PICKER_PANEL_HEIGHT};
use iced_color_picker::{PICKER_PANEL_WIDTH_RGBA, PICKER_PANEL_HEIGHT_RGBA};

// RGB
let state = ColorPickerState::from_color(Color::from_rgb(0.25, 0.55, 0.95));

// RGBA
let state = ColorPickerState::from_rgba(Color { r: 0.95, g: 0.40, b: 0.25, a: 0.6 });

// In your view
color_picker(&state)
    .border_radius(8.0)
    .show_sliders(true)
```

Handle `PickerMessage` in your update function and call `state.update(&message)` for all variants. For `CopyHex`, also write `state.hex()` to the clipboard and send back `PickerMessage::CopyConfirmed` after a short delay.
