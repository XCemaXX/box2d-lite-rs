use crate::body::Body;
use crate::contact::{Contact, Contacts, EdgeNumbers, Feature, MAX_CONTACT_POINT};
use crate::math_utils::{Mat22, Vec2};

#[derive(Debug, PartialEq, Default)]
enum Axis {
    #[default]
    FaceAX,
    FaceAY,
    FaceBX,
    FaceBY,
}

#[derive(Debug, Default, Copy, Clone)]
struct ClipVertex {
    v: Vec2,
    fp: Feature,
}

fn flip(fp: &mut Feature) {
    (fp.in_edge1, fp.in_edge2) = (fp.in_edge2, fp.in_edge1);
    (fp.out_edge1, fp.out_edge2) = (fp.out_edge2, fp.out_edge1);
}

/// `None` when fewer than two points are left: the caller needs both.
fn clip_segment_to_line(
    v_in: [ClipVertex; MAX_CONTACT_POINT],
    normal: Vec2,
    offset: f32,
    clip_edge: EdgeNumbers,
) -> Option<[ClipVertex; MAX_CONTACT_POINT]> {
    let mut v_out = [ClipVertex::default(); MAX_CONTACT_POINT];
    // Start with no output points
    let mut num_out = 0_usize;
    // Calculate the distance of end points to the line
    let distance0 = normal.dot(v_in[0].v) - offset;
    let distance1 = normal.dot(v_in[1].v) - offset;
    // If the points are behind the plane
    if distance0 <= 0.0 {
        v_out[num_out] = v_in[0];
        num_out += 1;
    }
    if distance1 <= 0.0 {
        v_out[num_out] = v_in[1];
        num_out += 1;
    }
    // If the points are on different sides of the plane
    if distance0 * distance1 < 0.0 {
        let interp = distance0 / (distance0 - distance1);
        v_out[num_out].v = v_in[0].v + (interp * (v_in[1].v - v_in[0].v));
        if distance0 > 0.0 {
            v_out[num_out].fp = v_in[0].fp;
            v_out[num_out].fp.in_edge1 = clip_edge;
            v_out[num_out].fp.in_edge2 = EdgeNumbers::NoEdge;
        } else {
            v_out[num_out].fp = v_in[1].fp;
            v_out[num_out].fp.out_edge1 = clip_edge;
            v_out[num_out].fp.out_edge2 = EdgeNumbers::NoEdge;
        }
        num_out += 1;
    }
    (num_out == MAX_CONTACT_POINT).then_some(v_out)
}

fn compute_incident_edge(
    h: Vec2,
    pos: Vec2,
    rot: Mat22,
    normal: Vec2,
) -> [ClipVertex; MAX_CONTACT_POINT] {
    let mut c = [ClipVertex::default(); MAX_CONTACT_POINT];
    // The normal is from the reference box. Convert it
    // to the incident boxe's frame and flip sign.
    let rot_t = rot.transpose();
    let n = -(rot_t * normal);
    let n_abs = n.abs();

    if n_abs.x > n_abs.y {
        if n.x >= 0.0 {
            c[0].v = Vec2::new(h.x, -h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge3;
            c[0].fp.out_edge2 = EdgeNumbers::Edge4;

            c[1].v = Vec2::new(h.x, h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge4;
            c[1].fp.out_edge2 = EdgeNumbers::Edge1;
        } else {
            c[0].v = Vec2::new(-h.x, h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge1;
            c[0].fp.out_edge2 = EdgeNumbers::Edge2;

            c[1].v = Vec2::new(-h.x, -h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge2;
            c[1].fp.out_edge2 = EdgeNumbers::Edge3;
        }
    } else {
        if n.y >= 0.0 {
            c[0].v = Vec2::new(h.x, h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge4;
            c[0].fp.out_edge2 = EdgeNumbers::Edge1;

            c[1].v = Vec2::new(-h.x, h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge1;
            c[1].fp.out_edge2 = EdgeNumbers::Edge2;
        } else {
            c[0].v = Vec2::new(-h.x, -h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge2;
            c[0].fp.out_edge2 = EdgeNumbers::Edge3;

            c[1].v = Vec2::new(h.x, -h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge3;
            c[1].fp.out_edge2 = EdgeNumbers::Edge4;
        }
    }

    c[0].v = pos + (rot * c[0].v);
    c[1].v = pos + (rot * c[1].v);
    c
}

pub fn collide(body_a: &Body, body_b: &Body) -> Contacts {
    // Setup
    let h_a = 0.5 * body_a.size;
    let h_b = 0.5 * body_b.size;

    let pos_a = body_a.position;
    let pos_b = body_b.position;

    let rot_a = body_a.rotation.mat();
    let rot_b = body_b.rotation.mat();

    let rot_at = rot_a.transpose();
    let rot_bt = rot_b.transpose();

    let dp = pos_b - pos_a;
    let da = rot_at * dp;
    let db = rot_bt * dp;

    let c = rot_at * rot_b;
    let abs_c = c.abs();
    let abs_ct = abs_c.transpose();

    // Box A faces
    let face_a = (da.abs() - h_a) - (abs_c * h_b);
    if face_a.x > 0.0 || face_a.y > 0.0 {
        return Contacts::default();
    }
    // Box B faces
    let face_b = (db.abs() - (abs_ct * h_a)) - h_b;
    if face_b.x > 0.0 || face_b.y > 0.0 {
        return Contacts::default();
    }

    // Find best axis
    // Box A faces
    let mut axis = Axis::FaceAX;
    let mut separation = face_a.x;
    let mut normal = if da.x > 0.0 { rot_a.col1 } else { -rot_a.col1 };

    const RELATIVE_TOL: f32 = 0.95;
    const ABSOLUTE_TOL: f32 = 0.01;

    if face_a.y > RELATIVE_TOL * separation + ABSOLUTE_TOL * h_a.y {
        axis = Axis::FaceAY;
        separation = face_a.y;
        normal = if da.y > 0.0 { rot_a.col2 } else { -rot_a.col2 };
    }

    // Box B faces
    if face_b.x > RELATIVE_TOL * separation + ABSOLUTE_TOL * h_b.x {
        axis = Axis::FaceBX;
        separation = face_b.x;
        normal = if db.x > 0.0 { rot_b.col1 } else { -rot_b.col1 };
    }

    if face_b.y > RELATIVE_TOL * separation + ABSOLUTE_TOL * h_b.y {
        axis = Axis::FaceBY;
        //separation = face_b.y; // unused
        normal = if db.y > 0.0 { rot_b.col2 } else { -rot_b.col2 };
    }

    // Setup clipping plane data based on the separating axis
    let (front_normal, side_normal);
    let incident_edge;
    let (front, neg_side, pos_side);
    let (neg_edge, pos_edge);

    // Compute the clipping lines and the line segment to be clipped.
    match axis {
        Axis::FaceAX => {
            front_normal = normal;
            front = pos_a.dot(front_normal) + h_a.x;
            side_normal = rot_a.col2;
            let side = pos_a.dot(side_normal);
            neg_side = -side + h_a.y;
            pos_side = side + h_a.y;
            neg_edge = EdgeNumbers::Edge3;
            pos_edge = EdgeNumbers::Edge1;
            incident_edge = compute_incident_edge(h_b, pos_b, rot_b, front_normal);
        }
        Axis::FaceAY => {
            front_normal = normal;
            front = pos_a.dot(front_normal) + h_a.y;
            side_normal = rot_a.col1;
            let side = pos_a.dot(side_normal);
            neg_side = -side + h_a.x;
            pos_side = side + h_a.x;
            neg_edge = EdgeNumbers::Edge2;
            pos_edge = EdgeNumbers::Edge4;
            incident_edge = compute_incident_edge(h_b, pos_b, rot_b, front_normal);
        }
        Axis::FaceBX => {
            front_normal = -normal;
            front = pos_b.dot(front_normal) + h_b.x;
            side_normal = rot_b.col2;
            let side = pos_b.dot(side_normal);
            neg_side = -side + h_b.y;
            pos_side = side + h_b.y;
            neg_edge = EdgeNumbers::Edge3;
            pos_edge = EdgeNumbers::Edge1;
            incident_edge = compute_incident_edge(h_a, pos_a, rot_a, front_normal);
        }
        Axis::FaceBY => {
            front_normal = -normal;
            front = pos_b.dot(front_normal) + h_b.y;
            side_normal = rot_b.col1;
            let side = pos_b.dot(side_normal);
            neg_side = -side + h_b.x;
            pos_side = side + h_b.x;
            neg_edge = EdgeNumbers::Edge2;
            pos_edge = EdgeNumbers::Edge4;
            incident_edge = compute_incident_edge(h_a, pos_a, rot_a, front_normal);
        }
    };

    // clip other face with 5 box planes (1 face plane, 4 edge planes)
    // Clip to box side 1
    let Some(clip_points1) = clip_segment_to_line(incident_edge, -side_normal, neg_side, neg_edge)
    else {
        return Contacts::default();
    };

    // Clip to negative box side 1
    let Some(clip_points2) = clip_segment_to_line(clip_points1, side_normal, pos_side, pos_edge)
    else {
        return Contacts::default();
    };

    // Now clipPoints2 contains the clipping points.
    // Due to roundoff, it is possible that clipping removes all points.

    let mut contacts = Contacts::default();
    for clip_point in &clip_points2 {
        let separation = front_normal.dot(clip_point.v) - front;

        if separation <= 0.0 {
            let mut feature = clip_point.fp;
            if axis == Axis::FaceBX || axis == Axis::FaceBY {
                flip(&mut feature)
            }
            contacts.push(Contact {
                separation,
                normal,
                // slide contact point onto reference face (easy to cull)
                position: clip_point.v - (separation * front_normal),
                feature,
                ..Default::default()
            });
        }
    }

    contacts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math_utils::Rotation;

    // A box corner pushed into a tilted thin plank. Both bodies are rotated,
    // so a wrong relative rotation in the SAT test hides the contact.
    #[test]
    fn rotated_corner_touches_tilted_plank() {
        let tilt: f32 = 0.25;
        let mut plank = Body::new_static(Vec2::new(110.0, 4.0), Vec2::ZERO);
        plank.rotation = Rotation::new(tilt);

        // Box corner reaches the plank face at normal distance ~8.9.
        let (along, normal) = (40.0, 8.0);
        let (sin, cos) = tilt.sin_cos();
        let position = Vec2::new(along * cos - normal * sin, along * sin + normal * cos);
        let mut b = Body::new_dynamic(Vec2::new(10.0, 10.0), 1.0, position);
        b.rotation = Rotation::new(tilt - 0.56);

        assert!(!collide(&plank, &b).is_empty());
        assert!(!collide(&b, &plank).is_empty());
    }
}
