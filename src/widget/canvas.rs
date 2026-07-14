use std::rc::Rc;

use crate::color::{DISC_ANGULAR_STEPS, DISC_RADIAL_STEPS, hsv_to_iced_color};
use crate::state::PickerMessage;
use crate::style::{Catalog, Status};
use iced::advanced::graphics::geometry;
use iced::advanced::graphics::gradient;
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Fill};
use iced::{Color, Point, Rectangle, Size, border};

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
    bar_border_radius: f32,
    class: Rc<Theme::Class<'a>>,
) -> Canvas<ValueBarProgram<'a, Theme>, PickerMessage, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    Canvas::new(ValueBarProgram {
        h,
        s,
        v,
        bar_border_radius,
        class,
    })
}

pub(crate) fn alpha_bar<'a, Theme, Renderer>(
    r: u8,
    g: u8,
    b: u8,
    a: f32,
    bar_border_radius: f32,
    class: Rc<Theme::Class<'a>>,
) -> Canvas<AlphaBarProgram<'a, Theme>, PickerMessage, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    Canvas::new(AlphaBarProgram {
        r,
        g,
        b,
        a,
        bar_border_radius,
        class,
    })
}

pub(crate) fn checkerboard_cells(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cell: f32,
    mut draw: impl FnMut(f32, f32, f32, f32, Color),
) {
    let light = Color {
        r: 0.80,
        g: 0.80,
        b: 0.80,
        a: 1.0,
    };
    let dark = Color {
        r: 0.60,
        g: 0.60,
        b: 0.60,
        a: 1.0,
    };
    let cols = (width / cell).ceil() as usize;
    let rows = (height / cell).ceil() as usize;
    for row in 0..rows {
        for col in 0..cols {
            let color = if (row + col) % 2 == 0 { light } else { dark };
            let cx = x + col as f32 * cell;
            let cy = y + row as f32 * cell;
            let cw = (cx + cell).min(x + width) - cx;
            let ch = (cy + cell).min(y + height) - cy;
            draw(cx, cy, cw, ch, color);
        }
    }
}

fn drag_mouse_interaction(
    dragging: bool,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> mouse::Interaction {
    if dragging {
        mouse::Interaction::Crosshair
    } else if cursor.is_over(bounds) {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

fn clamped_position_in(cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
    cursor.position().map(|pos| {
        Point::new(
            (pos.x - bounds.x).clamp(0.0, bounds.width),
            (pos.y - bounds.y).clamp(0.0, bounds.height),
        )
    })
}

fn handle_drag(
    dragging: &mut bool,
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    pick: impl Fn(Point) -> Option<PickerMessage>,
) -> Option<canvas::Action<PickerMessage>> {
    match event {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if let Some(pos) = cursor.position_in(bounds)
                && let Some(msg) = pick(pos)
            {
                *dragging = true;
                return Some(canvas::Action::publish(msg).and_capture());
            }
            None
        }
        canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if *dragging => {
            if let Some(pos) = clamped_position_in(cursor, bounds)
                && let Some(msg) = pick(pos)
            {
                return Some(canvas::Action::publish(msg).and_capture());
            }
            Some(canvas::Action::capture())
        }
        canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if *dragging => {
            *dragging = false;
            Some(canvas::Action::capture())
        }
        _ => None,
    }
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
        handle_drag(&mut state.dragging, event, bounds, cursor, |pos| {
            self.pos_to_hs(pos, size)
                .map(|(h, s)| PickerMessage::HueSatFromDisc { h, s })
        })
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
                .with_width(1.5),
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
        drag_mouse_interaction(state.dragging, bounds, cursor)
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
    pub bar_border_radius: f32,
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
        let bar_height = bounds.height.max(1.0);
        handle_drag(&mut state.dragging, event, bounds, cursor, |pos| {
            let v = (1.0 - (pos.y / bar_height).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            Some(PickerMessage::ValueFromBar(v))
        })
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

        if self.bar_border_radius > 0.0 {
            let r = border::Radius::from(self.bar_border_radius);
            let grad = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, h))
                .add_stop(0.0, hsv_to_iced_color(self.h, self.s, 1.0))
                .add_stop(1.0, hsv_to_iced_color(self.h, self.s, 0.0));
            frame.fill(
                &canvas::Path::rounded_rectangle(Point::ORIGIN, Size::new(w, h), r),
                Fill::from(grad),
            );
            let vy = (1.0 - self.v) * h;
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, vy), Point::new(w, vy)),
                canvas::Stroke::default()
                    .with_color(picker.value_indicator)
                    .with_width(4.0),
            );
            frame.stroke(
                &canvas::Path::rounded_rectangle(Point::ORIGIN, Size::new(w, h), r),
                canvas::Stroke::default()
                    .with_color(picker.canvas_frame)
                    .with_width(1.5),
            );
        } else {
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
                    .with_width(4.0),
            );
            frame.stroke_rectangle(
                Point::new(0.0, 0.0),
                Size::new(w, h),
                canvas::Stroke::default()
                    .with_color(picker.canvas_frame)
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        drag_mouse_interaction(state.dragging, bounds, cursor)
    }
}

pub(crate) struct AlphaBarProgram<'a, Theme: Catalog + 'a> {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
    pub bar_border_radius: f32,
    pub class: Rc<Theme::Class<'a>>,
}

impl<'a, Theme, Renderer> canvas::Program<PickerMessage, Theme, Renderer>
    for AlphaBarProgram<'a, Theme>
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
        let bar_height = bounds.height.max(1.0);
        handle_drag(&mut state.dragging, event, bounds, cursor, |pos| {
            let a = (1.0 - (pos.y / bar_height).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            let a_u8 = (a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            Some(PickerMessage::AlphaChanged(a_u8))
        })
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
        let rf = self.r as f32 / 255.0;
        let gf = self.g as f32 / 255.0;
        let bf = self.b as f32 / 255.0;

        if self.bar_border_radius > 0.0 {
            let r = self.bar_border_radius;
            let radius = border::Radius::from(r);
            checkerboard_cells(0.0, 0.0, w, h, 4.0, |x, y, cw, ch, color| {
                let in_tl = x < r && y < r;
                let in_tr = x + cw > w - r && y < r;
                let in_bl = x < r && y + ch > h - r;
                let in_br = x + cw > w - r && y + ch > h - r;
                if !in_tl && !in_tr && !in_bl && !in_br {
                    frame.fill_rectangle(Point::new(x, y), Size::new(cw, ch), color);
                }
            });
            let grad = gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, h))
                .add_stop(
                    0.0,
                    Color {
                        r: rf,
                        g: gf,
                        b: bf,
                        a: 1.0,
                    },
                )
                .add_stop(
                    1.0,
                    Color {
                        r: rf,
                        g: gf,
                        b: bf,
                        a: 0.0,
                    },
                );
            frame.fill(
                &canvas::Path::rounded_rectangle(Point::ORIGIN, Size::new(w, h), radius),
                Fill::from(grad),
            );
            let ay = (1.0 - self.a) * h;
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, ay), Point::new(w, ay)),
                canvas::Stroke::default()
                    .with_color(picker.alpha_indicator)
                    .with_width(4.0),
            );
            frame.stroke(
                &canvas::Path::rounded_rectangle(Point::ORIGIN, Size::new(w, h), radius),
                canvas::Stroke::default()
                    .with_color(picker.canvas_frame)
                    .with_width(1.5),
            );
        } else {
            checkerboard_cells(0.0, 0.0, w, h, 4.0, |x, y, cw, ch, color| {
                frame.fill_rectangle(Point::new(x, y), Size::new(cw, ch), color);
            });
            let steps = 32;
            let strip_h = h / steps as f32;
            for i in 0..steps {
                let a0 = 1.0 - i as f32 / steps as f32;
                let a1 = 1.0 - (i + 1) as f32 / steps as f32;
                let a_mid = (a0 + a1) * 0.5;
                let color = Color {
                    r: rf,
                    g: gf,
                    b: bf,
                    a: a_mid,
                };
                let y0 = i as f32 * strip_h;
                frame.fill_rectangle(Point::new(0.0, y0), Size::new(w, strip_h), color);
            }
            let ay = (1.0 - self.a) * h;
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, ay), Point::new(w, ay)),
                canvas::Stroke::default()
                    .with_color(picker.alpha_indicator)
                    .with_width(4.0),
            );
            frame.stroke_rectangle(
                Point::new(0.0, 0.0),
                Size::new(w, h),
                canvas::Stroke::default()
                    .with_color(picker.canvas_frame)
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        drag_mouse_interaction(state.dragging, bounds, cursor)
    }
}
