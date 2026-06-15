use crate::color::{hsv_to_rgb8, parse_rgb_hex, rgb8_to_hsv, sanitize_hex_field_input};

#[derive(Debug, Clone)]
pub enum PickerMessage {
    HueSatFromDisc { h: f32, s: f32 },
    ValueFromBar(f32),
    RedChanged(u8),
    GreenChanged(u8),
    BlueChanged(u8),
    HexEdited(String),
    CopyHex,
}

pub struct ColorPickerState {
    h: f32,
    s: f32,
    v: f32,
    hex_field: String,
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
            hex_field: format!("#{r:02X}{g:02X}{b:02X}"),
        }
    }

    pub fn to_color(&self) -> iced::Color {
        let (r, g, b) = hsv_to_rgb8(self.h, self.s, self.v);
        iced::Color::from_rgb8(r, g, b)
    }

    pub fn hsv(&self) -> (f32, f32, f32) {
        (self.h, self.s, self.v)
    }

    pub fn hex(&self) -> &str {
        &self.hex_field
    }

    pub(crate) fn rgb8(&self) -> (u8, u8, u8) {
        hsv_to_rgb8(self.h, self.s, self.v)
    }

    pub(crate) fn hex_field(&self) -> &str {
        &self.hex_field
    }

    fn sync_hex_field_from_rgb(&mut self) {
        let (r, g, b) = self.rgb8();
        self.hex_field = format!("#{r:02X}{g:02X}{b:02X}");
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
            PickerMessage::HexEdited(s) => {
                self.hex_field = sanitize_hex_field_input(s);
                if let Some((r, g, b)) = parse_rgb_hex(&self.hex_field) {
                    let (h, se, v) = rgb8_to_hsv(r, g, b);
                    self.h = h;
                    self.s = se;
                    self.v = v;
                    self.hex_field = format!("#{r:02X}{g:02X}{b:02X}");
                }
            }
            PickerMessage::CopyHex => {}
        }
    }
}
