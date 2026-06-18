//! Run with: `cargo run --example demo_rgb`

use iced::widget::container;
use iced::{Color, Element, Task, window};
use iced_color_picker::{
    ColorPickerState, PICKER_PANEL_HEIGHT, PICKER_PANEL_WIDTH, PickerMessage, color_picker,
};

const BORDER_RADIUS: f32 = 8.0;

struct Demo {
    picker: ColorPickerState,
}

fn main() -> iced::Result {
    iced::application(
        || Demo {
            picker: ColorPickerState::from_color(Color::from_rgb(0.25, 0.55, 0.95)),
        },
        update,
        view,
    )
    .title("iced_color_picker — RGB")
    .theme(iced::Theme::Dark)
    .window(window::Settings {
        size: iced::Size::new(PICKER_PANEL_WIDTH + 24.0, PICKER_PANEL_HEIGHT + 24.0),
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
            .on_copy(|hex| println!("copied: {hex}")),
    )
        .padding(12)
        .into()
}
