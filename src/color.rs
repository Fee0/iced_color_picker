// HSV (h,s,v in [0,1]) <-> RGB8

pub(crate) fn rgb8_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let d = max - min;
    let v = max;
    let s = if max <= 1e-5 { 0.0 } else { d / max };
    let h_deg = if d <= 1e-5 {
        0.0
    } else if (max - rf).abs() < 1e-5 {
        let mut hh = (gf - bf) / d * 60.0;
        if gf < bf {
            hh += 360.0;
        }
        hh
    } else if (max - gf).abs() < 1e-5 {
        ((bf - rf) / d + 2.0) * 60.0
    } else {
        ((rf - gf) / d + 4.0) * 60.0
    };
    let h = (h_deg / 360.0).rem_euclid(1.0);
    (h, s, v)
}

pub(crate) fn hsv_to_rgb8(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = (h.fract() + 1.0).fract();
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match (h * 6.0).floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

pub(crate) fn hsv_to_iced_color(h: f32, s: f32, v: f32) -> iced::Color {
    let (r, g, b) = hsv_to_rgb8(h, s, v);
    iced::Color::from_rgb8(r, g, b)
}

pub(crate) fn contrast_text_color(r: u8, g: u8, b: u8) -> iced::Color {
    let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luma > 140.0 {
        iced::Color::BLACK
    } else {
        iced::Color::WHITE
    }
}

pub(crate) const DISC_ANGULAR_STEPS: usize = 72;
pub(crate) const DISC_RADIAL_STEPS: usize = 36;

pub(crate) fn parse_rgb_hex(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim();
    let digits = s.strip_prefix('#').unwrap_or(s);
    if digits.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(digits.get(0..2)?, 16).ok()?;
    let g = u8::from_str_radix(digits.get(2..4)?, 16).ok()?;
    let b = u8::from_str_radix(digits.get(4..6)?, 16).ok()?;
    Some((r, g, b))
}

/// At most 7 characters: optional `#` plus up to 6 hex digits. Strips any other characters.
pub(crate) fn sanitize_hex_field_input(s: &str) -> String {
    const MAX_LEN_WITH_HASH: usize = 7;
    const MAX_LEN_PLAIN_HEX: usize = 6;
    let mut out = String::new();
    for c in s.chars() {
        if out.is_empty() && c == '#' {
            out.push('#');
            continue;
        }
        if c.is_ascii_hexdigit() {
            let cap = if out.starts_with('#') {
                MAX_LEN_WITH_HASH
            } else {
                MAX_LEN_PLAIN_HEX
            };
            if out.len() < cap {
                out.push(c);
            }
        }
    }
    out
}
