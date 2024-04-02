use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use image::RgbImage;
use imageproc::{drawing, rect::Rect};

pub fn render_text(width: u32, height: u32, text: String) -> RgbImage {
    // A FontSystem provides access to detected system fonts, create one per application
    let mut font_system = FontSystem::new();

    // A SwashCache stores rasterized glyphs, create one per application
    let mut swash_cache = SwashCache::new();

    // Text metrics indicate the font size and line height of a buffer
    let metrics = Metrics::new(50.0, 80.0);

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);

    // Borrow buffer together with the font system for more convenient method calls
    let mut buffer = buffer.borrow_with(&mut font_system);

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
    buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
        // Ignore any with alpha of zero
        if color.a() == 0 {
            return;
        }

        // Fill in your code here for drawing rectangles
        let rect = Rect::at(x, y).of_size(w, h);
        let (r, g, b, _a) = color.as_rgba_tuple();
        let pixel = image::Rgb::<u8>([r, g, b]);
        drawing::draw_hollow_rect_mut(&mut img, rect, pixel);
    });

    img
}
