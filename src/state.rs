use std::cell::Cell;

use crate::color::{
    hsv_to_rgb8, parse_rgb_hex, parse_rgba_hex, rgb8_to_hsv, sanitize_hex_field_input,
    sanitize_hex_field_input_rgba,
};

#[derive(Debug, Clone)]
pub enum PickerMessage {
    HueSatFromDisc { h: f32, s: f32 },
    ValueFromBar(f32),
    RedChanged(u8),
    GreenChanged(u8),
    BlueChanged(u8),
    AlphaChanged(u8),
    HexEdited(String),
    #[doc(hidden)]
    CopyHex,
}

pub struct ColorPickerState {
    h: f32,
    s: f32,
    v: f32,
    a: f32,
    alpha_enabled: bool,
    hex_field: String,
    copy_confirmed_until: Cell<Option<std::time::Instant>>,
}

impl ColorPickerState {
    pub fn from_color(c: iced::Color) -> Self {
        let r = (c.r * 255.0 + 0.5) as u8;
        let g = (c.g * 255.0 + 0.5) as u8;
        let b = (c.b * 255.0 + 0.5) as u8;
        let (h, s, v) = rgb8_to_hsv(r, g, b);
        Self {
            h,
            s,
            v,
            a: 1.0,
            alpha_enabled: false,
            hex_field: format!("#{r:02X}{g:02X}{b:02X}"),
            copy_confirmed_until: Cell::new(None),
        }
    }

    pub fn from_rgba(c: iced::Color) -> Self {
        let r = (c.r * 255.0 + 0.5) as u8;
        let g = (c.g * 255.0 + 0.5) as u8;
        let b = (c.b * 255.0 + 0.5) as u8;
        let a = (c.a * 255.0 + 0.5) as u8;
        let (h, s, v) = rgb8_to_hsv(r, g, b);
        Self {
            h,
            s,
            v,
            a: c.a.clamp(0.0, 1.0),
            alpha_enabled: true,
            hex_field: format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
            copy_confirmed_until: Cell::new(None),
        }
    }

    pub fn to_color(&self) -> iced::Color {
        let (r, g, b) = hsv_to_rgb8(self.h, self.s, self.v);
        iced::Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: self.a,
        }
    }

    pub fn hsv(&self) -> (f32, f32, f32) {
        (self.h, self.s, self.v)
    }

    pub fn hex(&self) -> &str {
        &self.hex_field
    }

    pub fn copy_confirmed(&self) -> bool {
        self.copy_confirmed_until
            .get()
            .map_or(false, |t| std::time::Instant::now() < t)
    }

    pub(crate) fn copy_confirmed_until(&self) -> Option<std::time::Instant> {
        self.copy_confirmed_until.get()
    }

    pub(crate) fn start_copy_confirmed(&self) {
        self.copy_confirmed_until.set(Some(
            std::time::Instant::now() + std::time::Duration::from_secs(1),
        ));
    }

    pub(crate) fn clear_copy_confirmed(&self) {
        self.copy_confirmed_until.set(None);
    }

    pub fn alpha_enabled(&self) -> bool {
        self.alpha_enabled
    }

    pub(crate) fn rgb8(&self) -> (u8, u8, u8) {
        hsv_to_rgb8(self.h, self.s, self.v)
    }

    pub(crate) fn alpha(&self) -> f32 {
        self.a
    }

    pub(crate) fn alpha_u8(&self) -> u8 {
        (self.a * 255.0 + 0.5).clamp(0.0, 255.0) as u8
    }

    pub(crate) fn hex_field(&self) -> &str {
        &self.hex_field
    }

    fn sync_hex_field_from_rgb(&mut self) {
        let (r, g, b) = self.rgb8();
        if self.alpha_enabled {
            let a = self.alpha_u8();
            self.hex_field = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
        } else {
            self.hex_field = format!("#{r:02X}{g:02X}{b:02X}");
        }
    }

    fn set_rgb8(&mut self, r: u8, g: u8, b: u8) {
        let (h, s, v) = rgb8_to_hsv(r, g, b);
        self.h = h;
        self.s = s;
        self.v = v;
        self.sync_hex_field_from_rgb();
    }

    pub fn update(&mut self, msg: &PickerMessage) {
        match msg {
            PickerMessage::HueSatFromDisc { h, s } => {
                self.h = *h;
                self.s = (*s).clamp(0.0, 1.0);
                self.sync_hex_field_from_rgb();
            }
            PickerMessage::ValueFromBar(v) => {
                self.v = (*v).clamp(0.0, 1.0);
                self.sync_hex_field_from_rgb();
            }
            PickerMessage::RedChanged(r) => {
                let (_, g, b) = self.rgb8();
                self.set_rgb8(*r, g, b);
            }
            PickerMessage::GreenChanged(g) => {
                let (r, _, b) = self.rgb8();
                self.set_rgb8(r, *g, b);
            }
            PickerMessage::BlueChanged(b) => {
                let (r, g, _) = self.rgb8();
                self.set_rgb8(r, g, *b);
            }
            PickerMessage::AlphaChanged(a) => {
                self.a = (*a as f32 / 255.0).clamp(0.0, 1.0);
                self.sync_hex_field_from_rgb();
            }
            PickerMessage::HexEdited(s) => {
                if self.alpha_enabled {
                    self.hex_field = sanitize_hex_field_input_rgba(s);
                    if let Some((r, g, b, a)) = parse_rgba_hex(&self.hex_field) {
                        let (h, se, v) = rgb8_to_hsv(r, g, b);
                        self.h = h;
                        self.s = se;
                        self.v = v;
                        self.a = a as f32 / 255.0;
                        self.hex_field = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
                    }
                } else {
                    self.hex_field = sanitize_hex_field_input(s);
                    if let Some((r, g, b)) = parse_rgb_hex(&self.hex_field) {
                        let (h, se, v) = rgb8_to_hsv(r, g, b);
                        self.h = h;
                        self.s = se;
                        self.v = v;
                        self.hex_field = format!("#{r:02X}{g:02X}{b:02X}");
                    }
                }
            }
            PickerMessage::CopyHex => {}
        }
    }
}
