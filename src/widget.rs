use std::rc::Rc;

use crate::canvas::{self, DISC_DIAMETER, VALUE_BAR_WIDTH};
use crate::style::{self, Catalog, Status as PickerStatus, StyleFn};
use crate::{ColorPickerState, PickerMessage, contrast_text_color};
use iced::advanced::graphics::geometry;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::svg::Handle;
use iced::widget::text_input;
use iced::widget::{Column, Row, button, container, slider, space, svg, text};
use iced::{
    Background, Border, Color, ContentFit, Element, Length, Rectangle, Shadow, Size, Vector,
};

/// Copy icon bytes from [`assets/svg/copy.svg`](../assets/svg/copy.svg). Stroke is recolored via `svg::Style::color`.
const COPY_ICON_SVG: &[u8] = include_bytes!("../assets/svg/copy.svg");

const DEFAULT_BORDER_RADIUS: f32 = 8.0;
const PICKER_VERTICAL_PADDING: f32 = 12.0;
const PREVIEW_HEIGHT: f32 = 52.0;
const SLIDER_BLOCK_HEIGHT: f32 = 90.0;

/// A theme-aware HSV color picker widget.
pub struct ColorPicker<
    'a,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> where
    Theme: Catalog,
{
    state: &'a ColorPickerState,
    border_radius: f32,
    width: Length,
    class: Rc<Theme::Class<'a>>,
    content: Element<'a, PickerMessage, Theme, Renderer>,
}

impl<'a, Theme, Renderer> ColorPicker<'a, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::renderer::Renderer + geometry::Renderer,
{
    /// Creates a [`ColorPicker`] for the given [`ColorPickerState`].
    pub fn new(state: &'a ColorPickerState) -> Self {
        Self {
            state,
            border_radius: DEFAULT_BORDER_RADIUS,
            width: Length::Fill,
            class: Rc::new(Theme::default()),
            content: space::horizontal().into(),
        }
    }

    /// Sets the corner radius used for the preview swatch and controls.
    #[must_use]
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Sets the width of the picker panel.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the style of the [`ColorPicker`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, PickerStatus) -> style::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = Rc::new((Box::new(style) as StyleFn<'a, Theme>).into());
        self
    }

    fn build_content(&self) -> Element<'a, PickerMessage, Theme, Renderer>
    where
        Theme: Catalog
            + button::Catalog
            + container::Catalog
            + text_input::Catalog
            + svg::Catalog
            + slider::Catalog
            + text::Catalog
            + 'a,
        for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
        for<'b> <Theme as container::Catalog>::Class<'b>: From<container::StyleFn<'b, Theme>>,
        for<'b> <Theme as text_input::Catalog>::Class<'b>: From<text_input::StyleFn<'b, Theme>>,
        for<'b> <Theme as svg::Catalog>::Class<'b>: From<svg::StyleFn<'b, Theme>>,
        Renderer: iced::advanced::renderer::Renderer
            + geometry::Renderer
            + iced::advanced::text::Renderer<Font = iced::Font>
            + iced::advanced::svg::Renderer
            + 'a,
    {
        let hex_border_radius = (self.border_radius * 0.5).max(1.0);
        let (r, g, b) = self.state.rgb8();
        let preview_color = self.state.to_color();
        let label_color = contrast_text_color(r, g, b);
        let border_radius = self.border_radius;

        let preview_inner = Row::new()
            .push(
                text_input("", self.state.hex_field())
                    .on_input(PickerMessage::HexEdited)
                    .font(iced::Font::MONOSPACE)
                    .size(14)
                    .padding([0, 2])
                    .width(Length::Fill)
                    .style({
                        let class = Rc::clone(&self.class);
                        move |theme: &Theme, status: text_input::Status| {
                            let picker = <Theme as Catalog>::style(
                                theme,
                                class.as_ref(),
                                text_input_picker_status(status),
                            );
                            text_input::Style {
                                background: Background::Color(Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                border: Border {
                                    radius: hex_border_radius.into(),
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
                    }),
            )
            .push({
                let copy_icon: Element<'a, PickerMessage, Theme, Renderer> =
                    svg(Handle::from_memory(COPY_ICON_SVG))
                        .width(Length::Fixed(20.0))
                        .height(Length::Fixed(20.0))
                        .content_fit(ContentFit::Contain)
                        .style(move |_theme: &Theme, _status: svg::Status| svg::Style {
                            color: Some(label_color),
                        })
                        .into();
                button(copy_icon)
                    .on_press(PickerMessage::CopyHex)
                    .padding(4)
                    .style(
                        move |_theme: &Theme, _status: button::Status| button::Style {
                            background: None,
                            text_color: label_color,
                            border: Border {
                                width: 0.0,
                                radius: border_radius.into(),
                                ..Border::default()
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                    )
            })
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let preview = container(preview_inner)
            .width(Length::Fill)
            .height(Length::Fixed(PREVIEW_HEIGHT))
            .padding([0, 10])
            .style({
                let class = Rc::clone(&self.class);
                move |theme: &Theme| {
                    let picker =
                        <Theme as Catalog>::style(theme, class.as_ref(), PickerStatus::Active);
                    container::Style {
                    background: Some(Background::Color(preview_color)),
                    border: Border {
                        color: picker.preview_border,
                        width: 1.0,
                        radius: border_radius.into(),
                    },
                        ..Default::default()
                    }
                }
            });

        let disc = canvas::saturation_disc(
            self.state.h,
            self.state.s,
            Rc::clone(&self.class),
        )
            .width(Length::Fixed(DISC_DIAMETER))
            .height(Length::Fixed(DISC_DIAMETER));

        let vbar = canvas::value_bar(
            self.state.h,
            self.state.s,
            self.state.v,
            Rc::clone(&self.class),
        )
            .width(Length::Fixed(VALUE_BAR_WIDTH))
            .height(Length::Fixed(DISC_DIAMETER));

        let sliders = Column::new()
            .push(channel_row("R", r, PickerMessage::RedChanged))
            .push(channel_row("G", g, PickerMessage::GreenChanged))
            .push(channel_row("B", b, PickerMessage::BlueChanged))
            .spacing(6);

        let disc_bar = Row::new()
            .push(disc)
            .push(vbar)
            .spacing(10)
            .align_y(iced::Alignment::Center);

        Column::new()
            .push(preview)
            .push(disc_bar)
            .push(sliders)
            .spacing(10)
            .padding(PICKER_VERTICAL_PADDING)
            .width(Length::Fill)
            .into()
    }
}

fn channel_row<'a, Theme, Renderer>(
    label: &'static str,
    value: u8,
    on_change: fn(u8) -> PickerMessage,
) -> Element<'a, PickerMessage, Theme, Renderer>
where
    Theme: slider::Catalog + text::Catalog + 'a,
    Renderer: iced::advanced::renderer::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
        + 'a,
{
    Row::new()
        .push(
            text(label)
                .size(13)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fixed(16.0)),
        )
        .push(slider(0..=255u8, value, on_change).width(Length::Fill))
        .push(
            text(format!("{value:3}"))
                .size(13)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fixed(32.0)),
        )
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
}

fn text_input_picker_status(status: text_input::Status) -> PickerStatus {
    match status {
        text_input::Status::Focused { .. } => PickerStatus::Focused,
        _ => PickerStatus::Active,
    }
}

impl<'a, Theme, Renderer> Widget<PickerMessage, Theme, Renderer> for ColorPicker<'a, Theme, Renderer>
where
    Theme: Catalog
        + button::Catalog
        + container::Catalog
        + text_input::Catalog
        + svg::Catalog
        + slider::Catalog
        + text::Catalog
        + 'a,
    for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
    for<'b> <Theme as container::Catalog>::Class<'b>: From<container::StyleFn<'b, Theme>>,
    for<'b> <Theme as text_input::Catalog>::Class<'b>: From<text_input::StyleFn<'b, Theme>>,
    for<'b> <Theme as svg::Catalog>::Class<'b>: From<svg::StyleFn<'b, Theme>>,
    Renderer: iced::advanced::renderer::Renderer
        + geometry::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
        + iced::advanced::svg::Renderer
        + 'a,
{
    fn diff(&self, _tree: &mut Tree) {}

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);
        self.content = self.build_content();
        tree.diff_children(std::slice::from_ref(&self.content));

        let node = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.loose(),
        );

        layout::Node::with_children(node.size(), vec![node])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, PickerMessage>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, PickerMessage, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Theme, Renderer> From<ColorPicker<'a, Theme, Renderer>>
    for Element<'a, PickerMessage, Theme, Renderer>
where
    Theme: Catalog
        + button::Catalog
        + container::Catalog
        + text_input::Catalog
        + svg::Catalog
        + slider::Catalog
        + text::Catalog
        + 'a,
    for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
    for<'b> <Theme as container::Catalog>::Class<'b>: From<container::StyleFn<'b, Theme>>,
    for<'b> <Theme as text_input::Catalog>::Class<'b>: From<text_input::StyleFn<'b, Theme>>,
    for<'b> <Theme as svg::Catalog>::Class<'b>: From<svg::StyleFn<'b, Theme>>,
    Renderer: iced::advanced::renderer::Renderer
        + geometry::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
        + iced::advanced::svg::Renderer
        + 'a,
{
    fn from(picker: ColorPicker<'a, Theme, Renderer>) -> Self {
        Element::new(picker)
    }
}

pub const PICKER_PANEL_HEIGHT: f32 = PICKER_VERTICAL_PADDING * 2.0
    + PREVIEW_HEIGHT
    + 10.0
    + DISC_DIAMETER
    + 10.0
    + SLIDER_BLOCK_HEIGHT;
