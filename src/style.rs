//! Styling for the [`ColorPicker`](crate::widget::ColorPicker) widget.
use iced::Theme;
use iced::widget::{button, svg, text_input};
use iced::{Background, Border, Color, Shadow, theme::Palette};
/// The appearance of a [`ColorPicker`](crate::widget::ColorPicker).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Preview swatch border.
    pub preview_border: Color,
    /// Hex field focus border and value bar indicator.
    pub focus_accent: Color,
    /// Hex field text selection.
    pub selection: Color,
    /// Disc ring and value bar frame.
    pub canvas_frame: Color,
    /// Saturation disc selector outer ring.
    pub selector_outer: Color,
    /// Saturation disc selector inner ring.
    pub selector_inner: Color,
    /// Value bar horizontal indicator (alias of focus accent by default).
    pub value_indicator: Color,
}
/// The style status of a [`ColorPicker`](crate::widget::ColorPicker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// Normal appearance.
    #[default]
    Active,
    /// Hex field or control has focus.
    Focused,
}
/// Snapshot context for child controls derived from the current color and layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PickerContext {
    pub label_color: Color,
    pub hex_border_radius: f32,
    pub border_radius: f32,
}
/// The style catalog of a [`ColorPicker`](crate::widget::ColorPicker).
pub trait Catalog {
    /// The style class of the catalog.
    type Class<'a>;
    /// The default class produced by the catalog.
    fn default<'a>() -> Self::Class<'a>;
    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}
/// Maps picker [`Style`] into child widget appearances.
pub(crate) trait CatalogExt: Catalog {
    fn hex_input_style(
        &self,
        class: &Self::Class<'_>,
        ctx: &PickerContext,
        status: text_input::Status,
    ) -> text_input::Style {
        let picker_status = match status {
            text_input::Status::Focused { .. } => Status::Focused,
            _ => Status::Active,
        };
        let picker = self.style(class, picker_status);
        let label_color = ctx.label_color;
        text_input::Style {
            background: Background::Color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
            border: Border {
                radius: ctx.hex_border_radius.into(),
                width: match status {
                    text_input::Status::Focused { .. } => 1.0,
                    _ => 0.0,
                },
                color: match status {
                    text_input::Status::Focused { .. } => picker.focus_accent,
                    _ => label_color.scale_alpha(0.45),
                },
            },
            icon: label_color,
            placeholder: label_color.scale_alpha(0.45),
            value: label_color,
            selection: picker.selection,
        }
    }
    fn copy_button_style(&self, ctx: &PickerContext) -> button::Style {
        button::Style {
            background: None,
            text_color: ctx.label_color,
            border: Border {
                width: 0.0,
                radius: ctx.border_radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
    fn copy_icon_style(&self, ctx: &PickerContext) -> svg::Style {
        svg::Style {
            color: Some(ctx.label_color),
        }
    }
}
impl<T: Catalog> CatalogExt for T {}
/// A styling function for a [`ColorPicker`](crate::widget::ColorPicker).
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;
impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}
/// The default [`Style`] of a [`ColorPicker`](crate::widget::ColorPicker).
#[must_use]
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let base = from_palette(&palette);
    match status {
        Status::Focused => {
            let extended = theme.extended_palette();
            Style {
                focus_accent: extended.primary.strong.color,
                value_indicator: extended.primary.strong.color,
                ..base
            }
        }
        Status::Active => base,
    }
}
/// A primary-styled [`ColorPicker`](crate::widget::ColorPicker).
#[must_use]
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    let extended = theme.extended_palette();
    let mut style = from_palette(&palette);
    style.focus_accent = extended.primary.base.color;
    style.value_indicator = extended.primary.base.color;
    style.selection = extended.primary.base.color.scale_alpha(0.35);
    match status {
        Status::Focused => Style {
            focus_accent: extended.primary.strong.color,
            value_indicator: extended.primary.strong.color,
            ..style
        },
        _ => style,
    }
}
fn from_palette(palette: &Palette) -> Style {
    let focus_accent = palette.primary;
    Style {
        preview_border: palette.text.scale_alpha(0.55),
        focus_accent,
        selection: focus_accent.scale_alpha(0.35),
        canvas_frame: palette.text.scale_alpha(0.45),
        selector_outer: palette.text,
        selector_inner: palette.background,
        value_indicator: focus_accent,
    }
}
