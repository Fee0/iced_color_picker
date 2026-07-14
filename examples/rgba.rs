//! Run with: `cargo run --example demo_rgba`

use iced::widget::container;
use iced::{Color, Element, Task, window};
use iced_color_picker::{
    ColorPickerState, PICKER_PANEL_HEIGHT_RGBA, PICKER_PANEL_WIDTH_RGBA, PickerMessage,
    color_picker,
};

const BORDER_RADIUS: f32 = 8.0;

struct Demo {
    picker: ColorPickerState,
}

fn main() -> iced::Result {
    iced::application(
        || Demo {
            picker: ColorPickerState::from_rgba(Color {
                r: 0.95,
                g: 0.40,
                b: 0.25,
                a: 0.6,
            }),
        },
        update,
        view,
    )
    .title("iced_color_picker — RGBA")
    .theme(iced::Theme::Dark)
    .window(window::Settings {
        size: iced::Size::new(
            PICKER_PANEL_WIDTH_RGBA + 24.0,
            PICKER_PANEL_HEIGHT_RGBA + 24.0,
        ),
        ..window::Settings::default()
    })
    .centered()
    .run()
}

fn update(state: &mut Demo, message: PickerMessage) -> Task<PickerMessage> {
    state.picker.update(&message);
    Task::none()
}

fn view(state: &Demo) -> Element<'_, PickerMessage> {
    container(
        color_picker(&state.picker)
            .border_radius(BORDER_RADIUS)
            .on_copy(|hex| println!("copied: {hex}"))
            .bar_border_radius(4.0),
    )
    .padding(12)
    .into()
}
