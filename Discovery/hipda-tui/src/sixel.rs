use image::{DynamicImage, GenericImageView};

fn closest_color(palette: &[[u8; 3]], target: [u8; 3]) -> usize {
    palette.iter().enumerate().min_by_key(|(_, c)| {
        let dr = c[0] as i32 - target[0] as i32;
        let dg = c[1] as i32 - target[1] as i32;
        let db = c[2] as i32 - target[2] as i32;
        (dr * dr + dg * dg + db * db) as u32
    }).map(|(i, _)| i).unwrap_or(0)
}

pub fn encode(img: &DynamicImage, max_w: u32, max_h: u32) -> Vec<u8> {
    let (orig_w, orig_h) = img.dimensions();
    let scale = (max_w as f64 / orig_w as f64).min(max_h as f64 / orig_h as f64).min(1.0);
    let w = (orig_w as f64 * scale) as u32;
    let h = (orig_h as f64 * scale * 0.5) as u32;
    let w = w.max(1); let h = h.max(1);
    let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    let mut buf = Vec::new();
    buf.extend_from_slice(b"\x1bPq");
    buf.extend_from_slice(format!("\"1;1;{};{}", w, h).as_bytes());
    let rgb = resized.to_rgb8();
    let pixels = rgb.as_raw();
    let mut palette: Vec<[u8; 3]> = Vec::with_capacity(256);
    let mut color_map = vec![0u8; pixels.len() / 3];
    for chunk in pixels.chunks_exact(3) {
        let c = [chunk[0], chunk[1], chunk[2]];
        let idx = palette.iter().position(|&p| p == c).unwrap_or_else(|| {
            if palette.len() < 256 { palette.push(c); palette.len() - 1 } else { closest_color(&palette, c) }
        });
        color_map[(chunk.as_ptr() as usize - pixels.as_ptr() as usize) / 3] = idx as u8;
    }
    for (ci, c) in palette.iter().enumerate() {
        let r = (c[0] as f64 * 100.0 / 255.0) as u8;
        let g = (c[1] as f64 * 100.0 / 255.0) as u8;
        let b = (c[2] as f64 * 100.0 / 255.0) as u8;
        buf.push(b'#');
        buf.extend_from_slice(format!("{};{};{}", r, g, b).as_bytes());
        for band in (0..h as usize).step_by(6) {
            let mut first = true;
            for x in 0..w as usize {
                let mut byte = 0u8;
                for dy in 0..6 {
                    let y = band + dy;
                    if y < h as usize && color_map[y * w as usize + x] == ci as u8 { byte |= 1 << dy; }
                }
                if byte != 0 {
                    if first { if band > 0 { buf.push(b'$'); } first = false; }
                    buf.push(63 + byte);
                } else if !first { first = true; }
            }
            if !first { buf.push(b'$'); }
        }
    }
    buf.extend_from_slice(b"\x1b\\");
    buf
}
