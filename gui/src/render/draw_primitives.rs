use crate::letterbox::Rect;
use crate::render::Vertex;
use crate::render::math;
use physics::primitives::{Line, Point, Rectangle, Triangle};

const RED_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const BLACK_COLOR: [f32; 3] = [0.0, 0.0, 0.0];
const BOX_CORNER_COLORS: [[f32; 3]; 4] = [
    [0.5, 0.0, 1.0],
    [0.9, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.9, 0.3],
];

/// Geometry of one frame. Cleared each frame; the vectors keep their capacity.
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Corners go around the quad, in either direction.
    fn push_quad(&mut self, corners: [[f32; 2]; 4], colors: [[f32; 3]; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend(
            corners
                .into_iter()
                .zip(colors)
                .map(|(position, color)| Vertex { position, color }),
        );
        self.indices.extend([0, 1, 3, 3, 1, 2].map(|i| base + i));
    }

    fn push_triangle(&mut self, points: [[f32; 2]; 3], color: [f32; 3]) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(points.map(|position| Vertex { position, color }));
        self.indices.extend([0, 1, 2].map(|i| base + i));
    }

    pub fn push_rect(&mut self, r: Rect, color: &wgpu::Color) {
        let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
        let color = color_as_array(color);
        self.push_quad([[x0, y0], [x1, y0], [x1, y1], [x0, y1]], [color; 4]);
    }

    pub fn push_bordered_rectangle(&mut self, r: &Rectangle, inner_color: &wgpu::Color) {
        const BORDER_WIDTH: f32 = 0.015;
        self.push_quad(math::get_corners(r), BOX_CORNER_COLORS);
        let inner = Rectangle {
            width: r.width - BORDER_WIDTH,
            height: r.height - BORDER_WIDTH,
            ..r.clone()
        };
        self.push_quad(math::get_corners(&inner), [color_as_array(inner_color); 4]);
    }

    pub fn push_point(&mut self, p: &Point) {
        const SIZE: f32 = 0.01;
        let (x, y) = (p.x, p.y);
        self.push_quad(
            [[x, y - SIZE], [x + SIZE, y - SIZE], [x + SIZE, y], [x, y]],
            [RED_COLOR; 4],
        );
    }

    pub fn push_line(&mut self, line: &Line) {
        const W: f32 = 0.005;
        let (p1, p2) = (&line.p1, &line.p2);
        let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
        let l = dx.hypot(dy);
        let u = dx * W * 0.5 / l;
        let v = dy * W * 0.5 / l;
        self.push_quad(
            [
                [p1.x + v, p1.y - u],
                [p1.x - v, p1.y + u],
                [p2.x - v, p2.y + u],
                [p2.x + v, p2.y - u],
            ],
            [BLACK_COLOR; 4],
        );
    }

    pub fn push_icon(&mut self, t: &Triangle) {
        self.push_triangle([t.p1.into(), t.p2.into(), t.p3.into()], BLACK_COLOR);
    }

    pub fn map_positions(&mut self, f: impl Fn([f32; 2]) -> [f32; 2]) {
        for v in &mut self.vertices {
            v.position = f(v.position);
        }
    }
}

fn color_as_array(color: &wgpu::Color) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}
