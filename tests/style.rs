use iced::Theme;
use iced_color_picker::{Status, default, primary};

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
