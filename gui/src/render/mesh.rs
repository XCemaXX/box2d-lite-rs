use crate::render::Vertex;

/// Geometry of one frame. Cleared each frame; the vectors keep their capacity.
#[derive(Default)]
pub struct Mesh {
    pub(super) vertices: Vec<Vertex>,
    pub(super) indices: Vec<u32>,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Corners go around the quad, in either direction.
    pub fn push_quad(&mut self, corners: [[f32; 2]; 4], colors: [[f32; 3]; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend(
            corners
                .into_iter()
                .zip(colors)
                .map(|(position, color)| Vertex { position, color }),
        );
        self.indices.extend([0, 1, 3, 3, 1, 2].map(|i| base + i));
    }

    pub fn push_triangle(&mut self, points: [[f32; 2]; 3], color: [f32; 3]) {
        let base = self.vertices.len() as u32;
        self.vertices
            .extend(points.map(|position| Vertex { position, color }));
        self.indices.extend([0, 1, 2].map(|i| base + i));
    }

    pub fn map_positions(&mut self, f: impl Fn([f32; 2]) -> [f32; 2]) {
        for v in &mut self.vertices {
            v.position = f(v.position);
        }
    }
}
