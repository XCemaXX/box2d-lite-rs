use crate::render::math;
use crate::primitives::{Rectangle, Point, Line, Triangle};
use crate::render::Vertex;

const RED_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const BLACK_COLOR: [f32; 3] = [0.0, 0.0, 0.0];

pub fn create_bordered_rectangle(r: Rectangle, index_start: &mut u16, inner_color: &wgpu::Color) -> (Vec<Vertex>, Vec<u16>) {
    let corners = math::get_corners(&r);
    const BORDER_WIDTH: f32 = 0.015;
    let mut r = r;
    r.width -= BORDER_WIDTH;
    r.height -= BORDER_WIDTH;
    let inner_corners = math::get_corners(&r);

    let vertices: Vec<Vertex> = vec![
        //outer square
        Vertex { position: corners[0], color: [0.5, 0.0, 1.0] },     // A
        Vertex { position: corners[1], color: [0.9, 1.0, 0.0] },    // B
        Vertex { position: corners[2], color: [0.0, 0.0, 1.0] },     // C
        Vertex { position: corners[3], color: [0.0, 0.9, 0.3] },      // D;

        //inner square
        Vertex { position: inner_corners[0], color: color_as_array(inner_color) },     
        Vertex { position: inner_corners[1], color: color_as_array(inner_color) },   
        Vertex { position: inner_corners[2], color: color_as_array(inner_color) },    
        Vertex { position: inner_corners[3], color: color_as_array(inner_color) },
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        //outer square
        i + 0, i + 1, i + 3,    // A, B, D
        i + 3, i + 1, i + 2,    // D, B, C
        //inner square
        i + 4, i + 5, i + 7,
        i + 7, i + 5, i + 6,
    ];
    *index_start += 8;
    return (vertices, indices);
}

pub fn create_point(p: &Point, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    let (x, y) = (p.x, p.y);
    const SIZE: f32  = 0.01;
    let color = RED_COLOR;
    let vertices: Vec<Vertex> = vec![
        //outer square
        Vertex { position: [x, y - SIZE], color },     // A
        Vertex { position: [x + SIZE, y - SIZE], color },    // B
        Vertex { position: [x + SIZE, y], color },     // C
        Vertex { position: [x, y], color },      // D;
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        //outer square
        i + 0, i + 1, i + 3,    // A, B, D
        i + 3, i + 1, i + 2,    // D, B, C
    ];
    *index_start += 4;
    return (vertices, indices);
}

pub fn create_line(line: &Line, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    const W: f32 = 0.005;
    let color: [f32; 3] = BLACK_COLOR;
    let p1 = &line.p1;
    let p2 = &line.p2;

    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let l = dx.hypot(dy);
    let u = dx * W * 0.5 / l;
    let v = dy * W * 0.5 / l;
    let vertices: Vec<Vertex> = vec![
        Vertex { position: [p1.x + v,  p1.y - u], color },
        Vertex { position: [p1.x - v,  p1.y + u], color },
        Vertex { position: [p2.x - v,  p2.y + u], color },
        Vertex { position: [p2.x + v,  p2.y - u], color },
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        i + 2, i + 1, i + 0,
        i + 2, i + 0, i + 3,
    ];
    *index_start += 4;
    return (vertices, indices);
}

pub fn create_triangle(t: &Triangle, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    let color = BLACK_COLOR;
    let vertices: Vec<Vertex> = vec![
        Vertex { position: t.p1.into(), color },     // A
        Vertex { position: t.p2.into(), color },    // B
        Vertex { position: t.p3.into(), color },     // C
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        i + 0, i + 1, i + 2,
    ];
    *index_start += 3;
    return (vertices, indices);
}

fn color_as_array(color: &wgpu::Color) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}