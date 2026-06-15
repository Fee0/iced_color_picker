//! Styling for the [`ColorPicker`](crate::widget::ColorPicker) widget.

use iced::Theme;
use iced::{Color, theme::Palette};

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
    /// Pointer is over the widget.
    Hovered,
    /// Widget is disabled.
    Disabled,
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
        Status::Hovered => Style {
            preview_border: palette.text.scale_alpha(0.7),
            ..base
        },
        Status::Disabled => Style {
            preview_border: palette.text.scale_alpha(0.25),
            focus_accent: palette.text.scale_alpha(0.35),
            canvas_frame: palette.text.scale_alpha(0.2),
            ..base
        },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dark_theme_active() {
        let theme = Theme::Dark;
        let style = default(&theme, Status::Active);
        assert_eq!(style.focus_accent, theme.palette().primary);
        assert!(style.preview_border.a > 0.0);
    }

    #[test]
    fn primary_dark_theme_focused() {
        let theme = Theme::Dark;
        let style = primary(&theme, Status::Focused);
        assert_eq!(
            style.focus_accent,
            theme.extended_palette().primary.strong.color
        );
    }
}
