use image::{DynamicImage, GenericImageView};

fn closest(palette: &[[u8; 3]], t: [u8; 3]) -> usize {
    palette.iter().enumerate().min_by_key(|(_, c)| {
        let dr = c[0] as i32 - t[0] as i32;
        let dg = c[1] as i32 - t[1] as i32;
        let db = c[2] as i32 - t[2] as i32;
        (dr * dr + dg * dg + db * db) as u32
    }).map(|(i, _)| i).unwrap_or(0)
}

/// Encode image as sixel. Returns empty vec if image is too small or encode fails.
pub fn encode(img: &DynamicImage, max_w: u32, max_h: u32) -> Vec<u8> {
    let (ow, oh) = img.dimensions();
    if ow == 0 || oh == 0 { return vec![]; }
    let scale = (max_w as f64 / ow as f64).min(max_h as f64 / oh as f64).min(1.0);
    let w = (ow as f64 * scale).max(1.0) as u32;
    let h = (oh as f64 * scale * 0.5).max(1.0) as u32;
    let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let pixels = rgb.as_raw();
    let total = pixels.len() / 3;
    if total == 0 { return vec![]; }

    // Quantize to 256-color palette
    let mut palette: Vec<[u8; 3]> = Vec::with_capacity(256);
    let mut map = vec![0u8; total];
    for (i, chunk) in pixels.chunks_exact(3).enumerate() {
        let c = [chunk[0], chunk[1], chunk[2]];
        if let Some(pos) = palette.iter().position(|p| *p == c) {
            map[i] = pos as u8;
        } else if palette.len() < 255 {
            palette.push(c);
            map[i] = (palette.len() - 1) as u8;
        } else {
            let pos = closest(&palette, c);
            map[i] = pos as u8;
        }
    }
    if palette.is_empty() { return vec![]; }

    let mut buf: Vec<u8> = Vec::with_capacity(total / 4 + 512);
    // Sixel introducer + raster attributes
    buf.extend_from_slice(b"\x1bP0;0;0q\"1;1;");
    buf.extend_from_slice(format!("{};{}", w, h).as_bytes());

    let rows = h as usize;
    let cols = w as usize;

    for (ci, c) in palette.iter().enumerate() {
        // Color definition: #R;G;B (0-100 scale)
        buf.push(b'#');
        let r = ((c[0] as u32 * 100 + 127) / 255) as u8;
        let g = ((c[1] as u32 * 100 + 127) / 255) as u8;
        let b = ((c[2] as u32 * 100 + 127) / 255) as u8;
        buf.extend_from_slice(format!("{};{};{}", r, g, b).as_bytes());

        // Encode all bands for this color
        for band_start in (0..rows).step_by(6) {
            if band_start > 0 {
                buf.push(b'-'); // next band (down 6 rows, col 0)
            }
            let band_end = (band_start + 6).min(rows);
            let mut run = false;
            for x in 0..cols {
                let mut byte: u8 = 0;
                for dy in 0..(band_end - band_start) {
                    let y = band_start + dy;
                    if map[y * cols + x] == ci as u8 {
                        byte |= 1 << dy;
                    }
                }
                if byte != 0 {
                    run = true;
                    buf.push(63 + byte);
                }
            }
            // End of band for this color: carriage return for next color
            if run {
                buf.push(b'$');
            }
        }
    }

    // Sixel terminator
    buf.extend_from_slice(b"\x1b\\");
    buf
}
