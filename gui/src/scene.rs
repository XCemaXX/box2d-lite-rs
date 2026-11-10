use crate::layout::BUTTONS;
use crate::letterbox::{Letterbox, Rect};
use crate::render::Mesh;
use physics::PhysicsState;
use physics::primitives::{Line, Point, Rectangle};

/// Clear color: the bars around the field.
pub const BARS: [f32; 3] = [0.5, 0.5, 0.5];
const GRAY_BACKGROUND: [f32; 3] = [0.66, 0.66, 0.66];
const GRAY_BUTTON: [f32; 3] = [0.77, 0.77, 0.77];
const RED: [f32; 3] = [1.0, 0.0, 0.0];
const BLACK: [f32; 3] = [0.0, 0.0, 0.0];
const BOX_CORNER_COLORS: [[f32; 3]; 4] = [
    [0.5, 0.0, 1.0],
    [0.9, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.9, 0.3],
];

/// Fills `mesh` with the frame in clip space of a viewport set to `letterbox.field()`.
pub fn build(mesh: &mut Mesh, letterbox: &Letterbox, physics: &PhysicsState) {
    mesh.clear();
    push_rect(mesh, letterbox.world(), GRAY_BACKGROUND);
    for b in BUTTONS {
        push_bordered_rectangle(mesh, &axis_aligned(b.rect), GRAY_BUTTON);
        let t = &b.icon;
        mesh.push_triangle([t.p1.into(), t.p2.into(), t.p3.into()], BLACK);
    }
    for r in physics.rectangles() {
        push_bordered_rectangle(mesh, &r, GRAY_BACKGROUND);
    }
    for p in physics.collide_points() {
        push_point(mesh, &p);
    }
    for l in physics.joint_lines() {
        push_line(mesh, &l);
    }
    mesh.map_positions(|p| letterbox.world_to_ndc(p));
}

fn push_rect(mesh: &mut Mesh, r: Rect, color: [f32; 3]) {
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    mesh.push_quad([[x0, y0], [x1, y0], [x1, y1], [x0, y1]], [color; 4]);
}

fn push_bordered_rectangle(mesh: &mut Mesh, r: &Rectangle, inner_color: [f32; 3]) {
    // Per side.
    const BORDER: f32 = 0.0075;
    mesh.push_quad(corners(r), BOX_CORNER_COLORS);
    let inner = Rectangle {
        width: r.width - 2.0 * BORDER,
        height: r.height - 2.0 * BORDER,
        ..r.clone()
    };
    mesh.push_quad(corners(&inner), [inner_color; 4]);
}

fn push_point(mesh: &mut Mesh, p: &Point) {
    const HALF: f32 = 0.005;
    let (x, y) = (p.x, p.y);
    mesh.push_quad(
        [
            [x - HALF, y - HALF],
            [x + HALF, y - HALF],
            [x + HALF, y + HALF],
            [x - HALF, y + HALF],
        ],
        [RED; 4],
    );
}

fn push_line(mesh: &mut Mesh, line: &Line) {
    const W: f32 = 0.005;
    let (p1, p2) = (&line.p1, &line.p2);
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    let l = dx.hypot(dy);
    if l == 0.0 {
        return;
    }
    let u = dx * W * 0.5 / l;
    let v = dy * W * 0.5 / l;
    mesh.push_quad(
        [
            [p1.x + v, p1.y - u],
            [p1.x - v, p1.y + u],
            [p2.x - v, p2.y + u],
            [p2.x + v, p2.y - u],
        ],
        [BLACK; 4],
    );
}

/// Corners around the rectangle: left bottom, right bottom, right top, left top.
fn corners(r: &Rectangle) -> [[f32; 2]; 4] {
    let (sin, cos) = r.rotation.sin_cos();
    let (hw, hh) = (r.width / 2.0, r.height / 2.0);
    [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)].map(|(x, y)| {
        [
            r.center.x + cos * x - sin * y,
            r.center.y + sin * x + cos * y,
        ]
    })
}

fn axis_aligned(r: Rect) -> Rectangle {
    Rectangle {
        center: Point {
            x: r.x + r.w / 2.0,
            y: r.y + r.h / 2.0,
        },
        width: r.w,
        height: r.h,
        rotation: 0.0,
    }
}
