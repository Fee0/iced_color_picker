use std::rc::Rc;

use crate::PickerMessage;
use crate::style::{Catalog, Status};
use crate::{DISC_ANGULAR_STEPS, DISC_RADIAL_STEPS, hsv_to_iced_color};
use iced::advanced::graphics::geometry;
use iced::mouse;
use iced::widget::canvas::{self, Canvas};
use iced::{Point, Rectangle, Size};

pub(crate) const DISC_DIAMETER: f32 = 200.0;
pub(crate) const VALUE_BAR_WIDTH: f32 = 28.0;

pub(crate) fn saturation_disc<'a, Theme, Renderer>(
    h: f32,
    s: f32,
    class: Rc<Theme::Class<'a>>,
) -> Canvas<SaturationDiscProgram<'a, Theme>, PickerMessage, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    Canvas::new(SaturationDiscProgram { h, s, class })
}

pub(crate) fn value_bar<'a, Theme, Renderer>(
    h: f32,
    s: f32,
    v: f32,
    class: Rc<Theme::Class<'a>>,
) -> Canvas<ValueBarProgram<'a, Theme>, PickerMessage, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    Canvas::new(ValueBarProgram { h, s, v, class })
}

#[derive(Default)]
pub(crate) struct DiscInteraction {
    dragging: bool,
}

pub(crate) struct SaturationDiscProgram<'a, Theme: Catalog + 'a> {
    pub h: f32,
    pub s: f32,
    pub class: Rc<Theme::Class<'a>>,
}

impl<'a, Theme: Catalog + 'a> SaturationDiscProgram<'a, Theme> {
    fn geometry(&self, size: Size) -> (Point, f32) {
        let cx = size.width * 0.5;
        let cy = size.height * 0.5;
        let radius = (size.width.min(size.height) * 0.5 - 4.0).max(1.0);
        (Point::new(cx, cy), radius)
    }

    fn pos_to_hs(&self, pos: Point, size: Size) -> Option<(f32, f32)> {
        let (c, radius) = self.geometry(size);
        let dx = pos.x - c.x;
        let dy = pos.y - c.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1e-3 {
            return Some((self.h, 0.0));
        }
        let s = (dist / radius).min(1.0);
        let h = (dy.atan2(dx) / std::f32::consts::TAU + 1.0).fract();
        Some((h, s))
    }
}

impl<'a, Theme, Renderer> canvas::Program<PickerMessage, Theme, Renderer>
    for SaturationDiscProgram<'a, Theme>
where
    Renderer: geometry::Renderer,
    Theme: Catalog + 'a,
{
    type State = DiscInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<PickerMessage>> {
        let size = bounds.size();
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some((h, s)) = self.pos_to_hs(pos, size) {
                        state.dragging = true;
                        return Some(
                            canvas::Action::publish(PickerMessage::HueSatFromDisc { h, s })
                                .and_capture(),
                        );
                    }
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some((h, s)) = self.pos_to_hs(pos, size) {
                        return Some(
                            canvas::Action::publish(PickerMessage::HueSatFromDisc { h, s })
                                .and_capture(),
                        );
                    }
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let picker = theme.style(self.class.as_ref(), Status::Active);
        let sz = bounds.size();
        let mut frame = canvas::Frame::new(renderer, sz);
        let (c, radius) = self.geometry(sz);
        let na = DISC_ANGULAR_STEPS;
        let nr = DISC_RADIAL_STEPS;
        let tau = std::f32::consts::TAU;

        for j in 0..nr {
            let r0 = radius * j as f32 / nr as f32;
            let r1 = radius * (j + 1) as f32 / nr as f32;
            let s_mid = (r0 + r1) / (2.0 * radius);
            for i in 0..na {
                let a0 = tau * i as f32 / na as f32;
                let a1 = tau * (i + 1) as f32 / na as f32;
                let h_mid = (i as f32 + 0.5) / na as f32;
                let color = hsv_to_iced_color(h_mid, s_mid, 1.0);
                let path = if r0 <= 0.01 {
                    let p1 = Point::new(c.x + r1 * a0.cos(), c.y + r1 * a0.sin());
                    let p2 = Point::new(c.x + r1 * a1.cos(), c.y + r1 * a1.sin());
                    canvas::Path::new(|b| {
                        b.move_to(c);
                        b.line_to(p1);
                        b.line_to(p2);
                        b.close();
                    })
                } else {
                    let p00 = Point::new(c.x + r0 * a0.cos(), c.y + r0 * a0.sin());
                    let p01 = Point::new(c.x + r0 * a1.cos(), c.y + r0 * a1.sin());
                    let p10 = Point::new(c.x + r1 * a0.cos(), c.y + r1 * a0.sin());
                    let p11 = Point::new(c.x + r1 * a1.cos(), c.y + r1 * a1.sin());
                    canvas::Path::new(|b| {
                        b.move_to(p00);
                        b.line_to(p10);
                        b.line_to(p11);
                        b.line_to(p01);
                        b.close();
                    })
                };
                frame.fill(&path, color);
            }
        }

        frame.stroke(
            &canvas::Path::circle(c, radius),
            canvas::Stroke::default()
                .with_color(picker.canvas_frame)
                .with_width(1.0),
        );

        let sel_a = self.h * tau;
        let sel_r = self.s * radius;
        let sx = c.x + sel_r * sel_a.cos();
        let sy = c.y + sel_r * sel_a.sin();
        let sel_pt = Point::new(sx, sy);
        frame.stroke(
            &canvas::Path::circle(sel_pt, 5.0),
            canvas::Stroke::default()
                .with_color(picker.selector_outer)
                .with_width(2.0),
        );
        frame.stroke(
            &canvas::Path::circle(sel_pt, 5.0),
            canvas::Stroke::default()
                .with_color(picker.selector_inner)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Crosshair
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Default)]
pub(crate) struct BarInteraction {
    dragging: bool,
}

pub(crate) struct ValueBarProgram<'a, Theme: Catalog + 'a> {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub class: Rc<Theme::Class<'a>>,
}

impl<'a, Theme, Renderer> canvas::Program<PickerMessage, Theme, Renderer>
    for ValueBarProgram<'a, Theme>
where
    Renderer: geometry::Renderer,
    Theme: Catalog + 'a,
{
    type State = BarInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<PickerMessage>> {
        let h = bounds.height.max(1.0);
        let pick_v = |y: f32| (1.0 - (y / h).clamp(0.0, 1.0)).clamp(0.0, 1.0);

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.dragging = true;
                    let v = pick_v(pos.y);
                    return Some(
                        canvas::Action::publish(PickerMessage::ValueFromBar(v)).and_capture(),
                    );
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let v = pick_v(pos.y);
                    return Some(
                        canvas::Action::publish(PickerMessage::ValueFromBar(v)).and_capture(),
                    );
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let picker = theme.style(self.class.as_ref(), Status::Active);
        let sz = bounds.size();
        let mut frame = canvas::Frame::new(renderer, sz);
        let w = sz.width;
        let h = sz.height;
        let steps = 32;
        let strip_h = h / steps as f32;

        for i in 0..steps {
            let v0 = 1.0 - i as f32 / steps as f32;
            let v1 = 1.0 - (i + 1) as f32 / steps as f32;
            let v_mid = (v0 + v1) * 0.5;
            let color = hsv_to_iced_color(self.h, self.s, v_mid);
            let y0 = i as f32 * strip_h;
            frame.fill_rectangle(Point::new(0.0, y0), Size::new(w, strip_h), color);
        }

        let vy = (1.0 - self.v) * h;
        frame.stroke(
            &canvas::Path::line(Point::new(0.0, vy), Point::new(w, vy)),
            canvas::Stroke::default()
                .with_color(picker.value_indicator)
                .with_width(2.0),
        );
        frame.stroke_rectangle(
            Point::new(0.0, 0.0),
            Size::new(w, h),
            canvas::Stroke::default()
                .with_color(picker.canvas_frame)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Crosshair
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
