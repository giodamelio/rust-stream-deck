use std::sync::OnceLock;

use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use image::RgbImage;
use imageproc::{drawing, rect::Rect};
use tokio::sync::Mutex;

pub fn font_renderer() -> &'static Mutex<FontRenderer> {
    static RENDERER: OnceLock<Mutex<FontRenderer>> = OnceLock::new();
    RENDERER.get_or_init(|| Mutex::new(FontRenderer::new()))
}

#[derive(Debug)]
pub struct FontRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl FontRenderer {
    pub fn new() -> Self {
        // A FontSystem provides access to detected system fonts, create one per application
        let font_system = FontSystem::new();

        // A SwashCache stores rasterized glyphs, create one per application
        let swash_cache = SwashCache::new();

        Self {
            font_system,
            swash_cache,
        }
    }

    pub fn render_text(&mut self, width: u32, height: u32, text: String) -> RgbImage {
        // Text metrics indicate the font size and line height of a buffer
        let metrics = Metrics::new(50.0, 80.0);

        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Borrow buffer together with the font system for more convenient method calls
        let mut buffer = buffer.borrow_with(&mut self.font_system);

        // Set a size for the text buffer, in pixels
        buffer.set_size(width as f32, height as f32);

        // Attributes indicate what font to choose
        let attrs = Attrs::new();

        // Add some text!
        buffer.set_text(&text, attrs, Shaping::Advanced);

        // Perform shaping as desired
        buffer.shape_until_scroll(true);

        // Create a default text color
        let text_color = Color::rgb(0xFF, 0xFF, 0xFF);

        // Create the image
        let mut img = image::ImageBuffer::new(width, height);

        // Render text
        buffer.draw(&mut self.swash_cache, text_color, |x, y, w, h, color| {
            // Ignore any with alpha of zero
            if color.a() == 0 {
                return;
            }

            // Draw rects, scaling the colors by the alpha to kinda blend it a bit
            let rect = Rect::at(x, y).of_size(w, h);
            let (r, g, b, a) = color.as_rgba_tuple();
            let scale = |c: u8| (c as i32 * a as i32 / 255).clamp(0, 255) as u8;
            let pixel = image::Rgb::<u8>([scale(r), scale(g), scale(b)]);
            drawing::draw_hollow_rect_mut(&mut img, rect, pixel);
        });

        img
    }
}
