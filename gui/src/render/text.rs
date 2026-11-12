//! Text in surface pixels, wrapped and clipped to a rect.
use crate::letterbox::Rect;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{Section, Text as Span, ab_glyph::FontRef},
};

pub struct Text {
    brush: TextBrush<FontRef<'static>>,
}

impl Text {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        (w, h): (u32, u32),
    ) -> Result<Self, String> {
        let font: &'static [u8] = include_bytes!("../../../fonts/DejaVuSans.ttf");
        let brush = BrushBuilder::using_font_bytes(font)
            .map_err(|e| format!("font: {e}"))?
            .build(device, w, h, format);
        Ok(Self { brush })
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, (w, h): (u32, u32)) {
        self.brush.resize_view(w as f32, h as f32, queue);
    }

    /// From the top-left corner of `area`, wrapped and clipped to it. `size` in surface pixels.
    /// The pass viewport must cover the whole surface.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        area: Rect,
        text: &str,
        size: f32,
    ) {
        let section = Section::default()
            .add_text(Span::new(text).with_scale(size))
            .with_screen_position((area.x, area.y))
            .with_bounds((area.w, area.h));
        match self.brush.queue(device, queue, [&section]) {
            Ok(()) => self.brush.draw(pass),
            Err(e) => log::warn!("Skip text: {e}"),
        }
    }
}
