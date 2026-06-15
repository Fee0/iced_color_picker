//! Iced color picker: saturation/hue disc, value bar, RGB sliders, hex field, and copy control.
//! This crate does not include a color-map grid or preset lists-only the picker UI and state.

mod color;
mod state;
mod style;
mod widget;

pub use state::{ColorPickerState, PickerMessage};
pub use style::{Catalog, Status, Style, StyleFn, default, primary};
pub use widget::{ColorPicker, PICKER_PANEL_HEIGHT, PICKER_PANEL_WIDTH, color_picker};
