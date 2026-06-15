mod canvas;

use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::color::contrast_text_color;
use crate::state::{ColorPickerState, PickerMessage};
use crate::style::{self, Catalog, CatalogExt, PickerContext, Status as PickerStatus, StyleFn};
use iced::advanced::graphics::geometry;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::tree::Tree;
use iced::advanced::widget::{Operation, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::widget::svg::Handle;
use iced::widget::{Column, Row, button, slider, space, svg, text, text_input};
use iced::{
    Background, Border, Color, ContentFit, Element, Length, Rectangle, Shadow, Size, Vector,
};

const COPY_ICON_SVG: &[u8] = include_bytes!("../assets/svg/copy.svg");

pub(crate) const DISC_DIAMETER: f32 = 200.0;
pub(crate) const VALUE_BAR_WIDTH: f32 = 28.0;
/// Recommended width for a panel containing the picker (disc + value bar + padding).
pub const PICKER_PANEL_WIDTH: f32 = DISC_DIAMETER + VALUE_BAR_WIDTH + 40.0;

const DEFAULT_BORDER_RADIUS: f32 = 8.0;
const PICKER_VERTICAL_PADDING: f32 = 12.0;
const PREVIEW_HEIGHT: f32 = 52.0;
const PREVIEW_HORIZONTAL_PADDING: f32 = 10.0;
const SLIDER_BLOCK_HEIGHT: f32 = 90.0;

struct PickerSnapshot {
    h: f32,
    s: f32,
    v: f32,
    hex_field: String,
    border_radius: f32,
    width: Length,
    class_revision: u64,
}

impl Hash for PickerSnapshot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.h.to_bits().hash(state);
        self.s.to_bits().hash(state);
        self.v.to_bits().hash(state);
        self.hex_field.hash(state);
        self.border_radius.to_bits().hash(state);
        match self.width {
            Length::Fill => 0u8.hash(state),
            Length::FillPortion(p) => {
                1u8.hash(state);
                p.hash(state);
            }
            Length::Shrink => 2u8.hash(state),
            Length::Fixed(f) => {
                3u8.hash(state);
                f.to_bits().hash(state);
            }
        }
        self.class_revision.hash(state);
    }
}

/// A theme-aware HSV color picker widget.
pub struct ColorPicker<'a, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    state: &'a ColorPickerState,
    border_radius: f32,
    width: Length,
    /// Shared style class (`StyleFn` is not `Clone`, so an `Rc` is used internally).
    class: Rc<Theme::Class<'a>>,
    class_revision: u64,
    content: Element<'a, PickerMessage, Theme, Renderer>,
    content_hash: u64,
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
            class_revision: 0,
            content: space::horizontal().into(),
            content_hash: 0,
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
        self.class_revision += 1;
        self
    }

    fn snapshot_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let snapshot = PickerSnapshot {
            h: self.state.h,
            s: self.state.s,
            v: self.state.v,
            hex_field: self.state.hex_field().to_string(),
            border_radius: self.border_radius,
            width: self.width,
            class_revision: self.class_revision,
        };

        let mut hasher = DefaultHasher::new();
        snapshot.hash(&mut hasher);
        hasher.finish()
    }

    fn build_content(&self) -> Element<'a, PickerMessage, Theme, Renderer>
    where
        Theme: Catalog
            + button::Catalog
            + text_input::Catalog
            + svg::Catalog
            + slider::Catalog
            + text::Catalog
            + 'a,
        for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
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
        let label_color = contrast_text_color(r, g, b);
        let ctx = PickerContext {
            label_color,
            hex_border_radius,
            border_radius: self.border_radius,
        };
        let class = Rc::clone(&self.class);

        let preview_inner = Row::new()
            .push(
                text_input("", self.state.hex_field())
                    .on_input(PickerMessage::HexEdited)
                    .font(iced::Font::MONOSPACE)
                    .size(14)
                    .padding([0, 2])
                    .width(Length::Fill)
                    .style({
                        let class = Rc::clone(&class);
                        move |theme: &Theme, status: text_input::Status| {
                            theme.hex_input_style(class.as_ref(), &ctx, status)
                        }
                    }),
            )
            .push({
                let copy_icon: Element<'a, PickerMessage, Theme, Renderer> =
                    svg(Handle::from_memory(COPY_ICON_SVG))
                        .width(Length::Fixed(20.0))
                        .height(Length::Fixed(20.0))
                        .content_fit(ContentFit::Contain)
                        .style(move |theme: &Theme, _status: svg::Status| {
                            theme.copy_icon_style(&ctx)
                        })
                        .into();
                button(copy_icon)
                    .on_press(PickerMessage::CopyHex)
                    .padding(4)
                    .style(move |theme: &Theme, _status: button::Status| {
                        theme.copy_button_style(&ctx)
                    })
            })
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        let preview_row = Row::new()
            .push(preview_inner.width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fixed(PREVIEW_HEIGHT))
            .padding([0.0, PREVIEW_HORIZONTAL_PADDING]);

        let disc = canvas::saturation_disc(self.state.h, self.state.s, Rc::clone(&class))
            .width(Length::Fixed(DISC_DIAMETER))
            .height(Length::Fixed(DISC_DIAMETER));

        let vbar = canvas::value_bar(self.state.h, self.state.s, self.state.v, Rc::clone(&class))
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
            .push(preview_row)
            .push(disc_bar)
            .push(sliders)
            .spacing(10)
            .padding(PICKER_VERTICAL_PADDING)
            .width(Length::Fill)
            .into()
    }

    fn rebuild_if_needed(&mut self, tree: &mut Tree)
    where
        Theme: Catalog
            + button::Catalog
            + text_input::Catalog
            + svg::Catalog
            + slider::Catalog
            + text::Catalog
            + 'a,
        for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
        for<'b> <Theme as text_input::Catalog>::Class<'b>: From<text_input::StyleFn<'b, Theme>>,
        for<'b> <Theme as svg::Catalog>::Class<'b>: From<svg::StyleFn<'b, Theme>>,
        Renderer: iced::advanced::renderer::Renderer
            + geometry::Renderer
            + iced::advanced::text::Renderer<Font = iced::Font>
            + iced::advanced::svg::Renderer
            + 'a,
    {
        let new_hash = self.snapshot_hash();

        if self.content_hash != new_hash {
            self.content_hash = new_hash;
            self.content = self.build_content();
            tree.diff_children(std::slice::from_ref(&self.content));
        }
    }
}

fn channel_row<'a, Theme, Renderer>(
    label: &'static str,
    value: u8,
    on_change: fn(u8) -> PickerMessage,
) -> Element<'a, PickerMessage, Theme, Renderer>
where
    Theme: slider::Catalog + text::Catalog + 'a,
    Renderer:
        iced::advanced::renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
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

fn draw_preview_chrome<Renderer>(
    renderer: &mut Renderer,
    preview_color: Color,
    border_color: Color,
    border_radius: f32,
    bounds: Rectangle,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::renderer::Renderer,
{
    if let Some(clipped) = bounds.intersection(viewport) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: clipped,
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: border_radius.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(preview_color),
        );
    }
}

impl<'a, Theme, Renderer> Widget<PickerMessage, Theme, Renderer>
    for ColorPicker<'a, Theme, Renderer>
where
    Theme: Catalog
        + button::Catalog
        + text_input::Catalog
        + svg::Catalog
        + slider::Catalog
        + text::Catalog
        + 'a,
    for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
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
        self.rebuild_if_needed(tree);

        let limits = limits.width(self.width);
        let node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());

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
        let content_layout = layout.children().next().unwrap();
        let picker_style =
            <Theme as Catalog>::style(theme, self.class.as_ref(), PickerStatus::Active);
        let preview_color = self.state.to_color();

        if let Some(preview_layout) = content_layout.children().next() {
            draw_preview_chrome(
                renderer,
                preview_color,
                picker_style.preview_border,
                self.border_radius,
                preview_layout.bounds(),
                viewport,
            );
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content_layout,
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
        + text_input::Catalog
        + svg::Catalog
        + slider::Catalog
        + text::Catalog
        + 'a,
    for<'b> <Theme as button::Catalog>::Class<'b>: From<button::StyleFn<'b, Theme>>,
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

/// Creates a [`ColorPicker`] widget for the given state.
pub fn color_picker<'a>(state: &'a ColorPickerState) -> ColorPicker<'a> {
    ColorPicker::new(state)
}
