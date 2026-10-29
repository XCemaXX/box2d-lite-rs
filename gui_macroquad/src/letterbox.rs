use macroquad::prelude::*;

/// Fits a world rectangle into the window with one scale for both axes.
/// The rest of the window is bars.
pub struct Letterbox {
    world: Rect,
    y_up: bool,
    field: Rect,
    scale: f32,
}

impl Letterbox {
    /// `world.x`, `world.y` is the min corner. `y_up`: world y grows up, as in physics.
    pub fn new(world: Rect, y_up: bool) -> Self {
        let mut letterbox = Self {
            world,
            y_up,
            field: Rect::default(),
            scale: 1.0,
        };
        letterbox.update();
        letterbox
    }

    /// Call once per frame: the window can change size at any time.
    pub fn update(&mut self) {
        let (screen_w, screen_h) = (screen_width(), screen_height());
        self.scale = (screen_w / self.world.w).min(screen_h / self.world.h);
        let (w, h) = (self.world.w * self.scale, self.world.h * self.scale);
        self.field = Rect::new((screen_w - w) / 2.0, (screen_h - h) / 2.0, w, h);
    }

    /// The world rectangle on screen, in pixels.
    pub fn field(&self) -> Rect {
        self.field
    }

    pub fn to_screen(&self, p: Vec2) -> Vec2 {
        let x = self.field.x + (p.x - self.world.x) * self.scale;
        let dy = (p.y - self.world.y) * self.scale;
        let y = if self.y_up {
            self.field.bottom() - dy
        } else {
            self.field.y + dy
        };
        vec2(x, y)
    }

    pub fn to_world(&self, p: Vec2) -> Vec2 {
        let x = self.world.x + (p.x - self.field.x) / self.scale;
        let dy = if self.y_up {
            self.field.bottom() - p.y
        } else {
            p.y - self.field.y
        };
        vec2(x, self.world.y + dy / self.scale)
    }

    pub fn len_to_screen(&self, len: f32) -> f32 {
        len * self.scale
    }

    /// Returns a top-left based rect, as macroquad draw functions expect.
    pub fn rect_to_screen(&self, r: Rect) -> Rect {
        let a = self.to_screen(r.point());
        let b = self.to_screen(r.point() + r.size());
        Rect::new(
            a.x.min(b.x),
            a.y.min(b.y),
            (b.x - a.x).abs(),
            (b.y - a.y).abs(),
        )
    }

    /// Cursor in world units, `None` outside the field.
    /// Touch works too: macroquad emulates the mouse with it.
    pub fn mouse_world(&self) -> Option<Vec2> {
        let p = Vec2::from(mouse_position());
        self.field.contains(p).then(|| self.to_world(p))
    }

    /// Scale for text rasterized at `font_size` to be `size` world units on screen.
    /// Keep `font_size` fixed: each new size adds glyphs to the font atlas.
    pub fn font_scale(&self, font_size: u16, size: f32) -> f32 {
        self.len_to_screen(size) / font_size as f32
    }

    /// Fills the bars and the field.
    pub fn clear(&self, bars: Color, field: Color) {
        clear_background(bars);
        let f = self.field;
        draw_rectangle(f.x, f.y, f.w, f.h, field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 16:9 world in a square window: bars at top and bottom.
    fn letterbox(y_up: bool) -> Letterbox {
        let world = Rect::new(-8.0, -4.5, 16.0, 9.0);
        let scale = 800.0 / 16.0;
        let h = 9.0 * scale;
        let field = Rect::new(0.0, (800.0 - h) / 2.0, 800.0, h);
        Letterbox {
            world,
            y_up,
            field,
            scale,
        }
    }

    #[test]
    fn corners() {
        let lb = letterbox(true);
        assert_eq!(lb.to_screen(vec2(-8.0, 4.5)), vec2(0.0, 175.0));
        assert_eq!(lb.to_screen(vec2(8.0, -4.5)), vec2(800.0, 625.0));
        let lb = letterbox(false);
        assert_eq!(lb.to_screen(vec2(-8.0, -4.5)), vec2(0.0, 175.0));
    }

    #[test]
    fn round_trip() {
        for y_up in [true, false] {
            let lb = letterbox(y_up);
            let p = vec2(1.25, -3.0);
            assert!((lb.to_world(lb.to_screen(p)) - p).length() < 1e-4);
        }
    }

    #[test]
    fn rect_is_top_left_based() {
        let lb = letterbox(true);
        let r = lb.rect_to_screen(Rect::new(0.0, 0.0, 2.0, 1.0));
        assert_eq!(r, Rect::new(400.0, 350.0, 100.0, 50.0));
    }
}
