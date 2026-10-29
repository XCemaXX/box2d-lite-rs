/// Axis-aligned rectangle; `x`, `y` is the min corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

/// Fits a world rectangle into the surface with one scale for both axes.
/// The rest of the surface is bars. Screen space is surface pixels, y down.
pub struct Letterbox {
    world: Rect,
    y_up: bool,
    field: Rect,
}

impl Letterbox {
    /// `y_up`: world y grows up, as in physics.
    pub fn new(world: Rect, y_up: bool) -> Self {
        Self {
            world,
            y_up,
            field: world,
        }
    }

    /// Call on every surface resize.
    pub fn resize(&mut self, screen_w: f32, screen_h: f32) {
        let scale = (screen_w / self.world.w).min(screen_h / self.world.h);
        let (w, h) = (self.world.w * scale, self.world.h * scale);
        self.field = Rect::new((screen_w - w) / 2.0, (screen_h - h) / 2.0, w, h);
    }

    pub fn world(&self) -> Rect {
        self.world
    }

    /// The world rectangle on screen, in pixels. Use it as the render viewport.
    pub fn field(&self) -> Rect {
        self.field
    }

    /// World point to clip space of a viewport set to `field()`.
    pub fn world_to_ndc(&self, p: [f32; 2]) -> [f32; 2] {
        let x = (p[0] - self.world.x) / self.world.w * 2.0 - 1.0;
        let y = (p[1] - self.world.y) / self.world.h * 2.0 - 1.0;
        [x, if self.y_up { y } else { -y }]
    }

    /// Screen pixel to world point, `None` outside the field.
    pub fn to_world(&self, x: f32, y: f32) -> Option<[f32; 2]> {
        if !self.field.contains(x, y) {
            return None;
        }
        let u = (x - self.field.x) / self.field.w;
        let v = (y - self.field.y) / self.field.h;
        let v = if self.y_up { 1.0 - v } else { v };
        Some([
            self.world.x + u * self.world.w,
            self.world.y + v * self.world.h,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 16:9 world in a square surface: bars at top and bottom.
    fn letterbox(y_up: bool) -> Letterbox {
        let mut lb = Letterbox::new(Rect::new(-8.0, -4.5, 16.0, 9.0), y_up);
        lb.resize(800.0, 800.0);
        lb
    }

    #[test]
    fn field_is_centered() {
        assert_eq!(letterbox(true).field(), Rect::new(0.0, 175.0, 800.0, 450.0));
    }

    #[test]
    fn to_world_corners() {
        let lb = letterbox(true);
        assert_eq!(lb.to_world(0.0, 175.0), Some([-8.0, 4.5]));
        assert_eq!(lb.to_world(800.0, 625.0), Some([8.0, -4.5]));
        assert_eq!(lb.to_world(400.0, 100.0), None);
        let lb = letterbox(false);
        assert_eq!(lb.to_world(0.0, 175.0), Some([-8.0, -4.5]));
    }

    #[test]
    fn world_to_ndc_corners() {
        let lb = letterbox(true);
        assert_eq!(lb.world_to_ndc([-8.0, -4.5]), [-1.0, -1.0]);
        assert_eq!(lb.world_to_ndc([8.0, 4.5]), [1.0, 1.0]);
        assert_eq!(letterbox(false).world_to_ndc([-8.0, -4.5]), [-1.0, 1.0]);
    }
}
