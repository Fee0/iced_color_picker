//! Run with: `cargo run --example demo`

use iced::widget::container;
use iced::{clipboard, window, Color, Element, Task};
use iced_color_picker::{color_picker, ColorPickerState, PickerMessage, PICKER_PANEL_WIDTH};

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
    .title("iced_color_picker demo")
    .theme(iced::Theme::Dark)
    .window(window::Settings {
        size: iced::Size::new(PICKER_PANEL_WIDTH + 24.0, 480.0),
        ..window::Settings::default()
    })
    .centered()
    .run()
}

fn update(state: &mut Demo, message: PickerMessage) -> Task<PickerMessage> {
    match message {
        PickerMessage::CopyHex => {
            state.picker.update(&PickerMessage::CopyHex);
            let hex = color_to_hex(state.picker.to_color());
            clipboard::write(hex)
        }
        _ => {
            state.picker.update(&message);
            Task::none()
        }
    }
}

fn view(state: &Demo) -> Element<'_, PickerMessage> {
    container(
        color_picker(&state.picker).border_radius(BORDER_RADIUS),
    )
    .padding(12)
    .into()
}

fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0 + 0.5) as u8;
    let g = (c.g * 255.0 + 0.5) as u8;
    let b = (c.b * 255.0 + 0.5) as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}
