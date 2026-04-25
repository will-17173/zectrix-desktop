use crate::plugin_output::TextImageStyle;
use fontdue::Font;
use image::{ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;
const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/NotoSansSC-VF.ttf");

pub fn render_text_to_png(text: &str, style: &TextImageStyle) -> anyhow::Result<Vec<u8>> {
    let font = Font::from_bytes(FONT_BYTES as &[u8], fontdue::FontSettings::default())
        .map_err(|err| anyhow::anyhow!("failed to load text image font: {err}"))?;
    let font_size = style.font_size as f32;
    let padding = style.padding.min(WIDTH / 2).min(HEIGHT / 2) as f32;
    let content_width = (WIDTH as f32 - (padding * 2.0)).max(1.0);
    let content_height = (HEIGHT as f32 - (padding * 2.0)).max(1.0);
    let line_metrics = font
        .horizontal_line_metrics(font_size)
        .ok_or_else(|| anyhow::anyhow!("font does not provide horizontal line metrics"))?;
    let line_height = (font_size * style.line_height).max(line_metrics.new_line_size).max(1.0);
    let max_lines = ((content_height / line_height).floor() as usize).max(1);
    let mut lines = wrap_text(&font, text, content_width, font_size);
    let was_truncated = lines.len() > max_lines;

    if was_truncated {
        lines.truncate(max_lines);
        if let Some(last_line) = lines.last_mut() {
            *last_line = append_ellipsis(&font, last_line, content_width, font_size);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    let total_text_height = line_height * lines.len() as f32;
    let top = match style.vertical_align.as_str() {
        "middle" => padding + ((content_height - total_text_height).max(0.0) / 2.0),
        _ => padding,
    };

    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));

    for (index, line) in lines.iter().enumerate() {
        let line_width = text_width(&font, line, font_size);
        let start_x = match style.align.as_str() {
            "center" => padding + ((content_width - line_width).max(0.0) / 2.0),
            "right" => padding + (content_width - line_width).max(0.0),
            _ => padding,
        };
        let baseline_y = top + (index as f32 * line_height) + line_metrics.ascent;

        draw_line(
            &mut image,
            &font,
            line,
            start_x.round() as i32,
            baseline_y.round() as i32,
            font_size,
        );
    }

    let mut cursor = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).write_to(&mut cursor, ImageFormat::Png)?;

    Ok(cursor.into_inner())
}

fn wrap_text(font: &Font, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0.0;

        for ch in paragraph.chars() {
            if ch == '\r' {
                continue;
            }

            let glyph_width = glyph_advance(font, ch, font_size);

            if current.is_empty() || current_width + glyph_width <= max_width {
                current.push(ch);
                current_width += glyph_width;
                continue;
            }

            lines.push(std::mem::take(&mut current));
            current.push(ch);
            current_width = glyph_width;
        }

        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn text_width(font: &Font, text: &str, font_size: f32) -> f32 {
    text.chars()
        .filter(|ch| *ch != '\r')
        .map(|ch| glyph_advance(font, ch, font_size))
        .sum()
}

fn trim_to_width(font: &Font, text: &str, max_width: f32, font_size: f32) -> String {
    let mut trimmed = text.to_string();

    while !trimmed.is_empty() && text_width(font, &trimmed, font_size) > max_width {
        trimmed.pop();
    }

    trimmed
}

fn append_ellipsis(font: &Font, line: &str, max_width: f32, font_size: f32) -> String {
    let ellipsis = "...";
    let ellipsis_width = text_width(font, ellipsis, font_size);

    if ellipsis_width >= max_width {
        return trim_to_width(font, ellipsis, max_width, font_size);
    }

    let trimmed = trim_to_width(font, line, max_width - ellipsis_width, font_size);
    format!("{trimmed}{ellipsis}")
}

fn draw_line(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    font: &Font,
    text: &str,
    start_x: i32,
    baseline_y: i32,
    font_size: f32,
) {
    let mut cursor_x = start_x as f32;

    for ch in text.chars() {
        if ch == '\r' {
            continue;
        }

        let render_char = if ch == '\t' { ' ' } else { ch };
        let (metrics, bitmap) = font.rasterize(render_char, font_size);
        let glyph_x = cursor_x.round() as i32 + metrics.xmin;
        let glyph_y = baseline_y - metrics.height as i32 - metrics.ymin;

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let alpha = bitmap[y * metrics.width + x];
                if alpha == 0 {
                    continue;
                }

                let pixel_x = glyph_x + x as i32;
                let pixel_y = glyph_y + y as i32;

                if pixel_x < 0
                    || pixel_y < 0
                    || pixel_x >= WIDTH as i32
                    || pixel_y >= HEIGHT as i32
                {
                    continue;
                }

                let pixel = image.get_pixel_mut(pixel_x as u32, pixel_y as u32);
                let inv_alpha = 255u16 - u16::from(alpha);

                for channel in 0..3 {
                    pixel.0[channel] = ((u16::from(pixel.0[channel]) * inv_alpha) / 255) as u8;
                }
                pixel.0[3] = 255;
            }
        }

        cursor_x += metrics.advance_width;
    }
}

fn glyph_advance(font: &Font, ch: char, font_size: f32) -> f32 {
    match ch {
        '\t' => font.metrics(' ', font_size).advance_width * 4.0,
        _ => font.metrics(ch, font_size).advance_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_output::TextImageStyle;

    #[test]
    fn renders_text_to_png_bytes() {
        let png = render_text_to_png("第一行\n第二行", &TextImageStyle::default()).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
        assert!(png.len() > 1000);
    }

    #[test]
    fn accepts_center_alignment() {
        let style = TextImageStyle {
            align: "center".into(),
            ..TextImageStyle::default()
        };

        let png = render_text_to_png("居中", &style).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
    }

    #[test]
    fn truncates_long_text_without_failing() {
        let text = "很长的文字".repeat(500);

        let png = render_text_to_png(&text, &TextImageStyle::default()).unwrap();

        assert!(png.starts_with(&[137, 80, 78, 71]));
    }
}