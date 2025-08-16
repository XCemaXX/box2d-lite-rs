use crate::body::Body;
use crate::contact::{Contact, EdgeNumbers, Feature, MAX_CONTACT_POINT};
use crate::math_utils::{dot, Mat22, Vec2};

#[derive(Debug, PartialEq)]
enum Axis {
    FaceAX,
    FaceAY,
    FaceBX,
    FaceBY,
}

impl Default for Axis {
    fn default() -> Self {
        Axis::FaceAX
    }
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

fn clip_segment_to_line(
    v_out: &mut [ClipVertex; MAX_CONTACT_POINT],
    v_in: [ClipVertex; MAX_CONTACT_POINT],
    normal: Vec2,
    offset: f32,
    clip_edge: EdgeNumbers,
) -> usize {
    // Start with no output points
    let mut num_out = 0_usize;
    // Calculate the distance of end points to the line
    let distance0 = dot(normal, v_in[0].v) - offset;
    let distance1 = dot(normal, v_in[1].v) - offset;
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
    num_out
}

fn compute_incident_edge(
    c: &mut [ClipVertex; MAX_CONTACT_POINT],
    h: Vec2,
    pos: Vec2,
    rot: Mat22,
    normal: Vec2,
) {
    // The normal is from the reference box. Convert it
    // to the incident boxe's frame and flip sign.
    let rot_t = rot.transpose();
    let n = -(rot_t * normal);
    let n_abs = n.abs();

    if n_abs.x > n_abs.y {
        if f32::signum(n.x) > 0.0 {
            c[0].v.set(h.x, -h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge3;
            c[0].fp.out_edge2 = EdgeNumbers::Edge4;

            c[1].v.set(h.x, h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge4;
            c[1].fp.out_edge2 = EdgeNumbers::Edge1;
        } else {
            c[0].v.set(-h.x, h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge1;
            c[0].fp.out_edge2 = EdgeNumbers::Edge2;

            c[1].v.set(-h.x, -h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge2;
            c[1].fp.out_edge2 = EdgeNumbers::Edge3;
        }
    } else {
        if f32::signum(n.y) > 0.0 {
            c[0].v.set(h.x, h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge4;
            c[0].fp.out_edge2 = EdgeNumbers::Edge1;

            c[1].v.set(-h.x, h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge1;
            c[1].fp.out_edge2 = EdgeNumbers::Edge2;
        } else {
            c[0].v.set(-h.x, -h.y);
            c[0].fp.in_edge2 = EdgeNumbers::Edge2;
            c[0].fp.out_edge2 = EdgeNumbers::Edge3;

            c[1].v.set(h.x, -h.y);
            c[1].fp.in_edge2 = EdgeNumbers::Edge3;
            c[1].fp.out_edge2 = EdgeNumbers::Edge4;
        }
    }

    c[0].v = pos + (rot * c[0].v);
    c[1].v = pos + (rot * c[1].v);
}

pub fn collide(contacts: &mut [Contact; MAX_CONTACT_POINT], body_a: &Body, body_b: &Body) -> usize {
    // Setup
    let h_a = 0.5 * body_a.width;
    let h_b = 0.5 * body_b.width;

    let pos_a = body_a.position;
    let pos_b = body_b.position;

    let rot_a = Mat22::from_angle(body_a.rotation);
    let rot_b = Mat22::from_angle(body_b.rotation);

    let rot_at = rot_a.transpose();
    let rot_bt = rot_b.transpose();

    let dp = pos_b - pos_a;
    let da = rot_at * dp;
    let db = rot_bt * dp;

    let c = rot_a * rot_b;
    let abs_c = c.abs();
    let abs_ct = abs_c.transpose();

    // Box A faces
    let face_a = (da.abs() - h_a) - (abs_c * h_b);
    if face_a.x > 0.0 || face_a.y > 0.0 {
        return 0;
    }
    // Box B faces
    let face_b = (db.abs() - (abs_ct * h_a)) - h_b;
    if face_b.x > 0.0 || face_b.y > 0.0 {
        return 0;
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
    let mut incident_edge: [ClipVertex; MAX_CONTACT_POINT] = Default::default();
    let (front, neg_side, pos_side);
    let (neg_edge, pos_edge);

    // Compute the clipping lines and the line segment to be clipped.
    match axis {
        Axis::FaceAX => {
            front_normal = normal;
            front = dot(pos_a, front_normal) + h_a.x;
            side_normal = rot_a.col2;
            let side = dot(pos_a, side_normal);
            neg_side = -side + h_a.y;
            pos_side = side + h_a.y;
            neg_edge = EdgeNumbers::Edge3;
            pos_edge = EdgeNumbers::Edge1;
            compute_incident_edge(&mut incident_edge, h_b, pos_b, rot_b, front_normal);
        }
        Axis::FaceAY => {
            front_normal = normal;
            front = dot(pos_a, front_normal) + h_a.y;
            side_normal = rot_a.col1;
            let side = dot(pos_a, side_normal);
            neg_side = -side + h_a.x;
            pos_side = side + h_a.x;
            neg_edge = EdgeNumbers::Edge2;
            pos_edge = EdgeNumbers::Edge4;
            compute_incident_edge(&mut incident_edge, h_b, pos_b, rot_b, front_normal);
        }
        Axis::FaceBX => {
            front_normal = -normal;
            front = dot(pos_b, front_normal) + h_b.x;
            side_normal = rot_b.col2;
            let side = dot(pos_b, side_normal);
            neg_side = -side + h_b.y;
            pos_side = side + h_b.y;
            neg_edge = EdgeNumbers::Edge3;
            pos_edge = EdgeNumbers::Edge1;
            compute_incident_edge(&mut incident_edge, h_a, pos_a, rot_a, front_normal);
        }
        Axis::FaceBY => {
            front_normal = -normal;
            front = dot(pos_b, front_normal) + h_b.y;
            side_normal = rot_b.col1;
            let side = dot(pos_b, side_normal);
            neg_side = -side + h_b.x;
            pos_side = side + h_b.x;
            neg_edge = EdgeNumbers::Edge2;
            pos_edge = EdgeNumbers::Edge4;
            compute_incident_edge(&mut incident_edge, h_a, pos_a, rot_a, front_normal);
        }
    };

    // clip other face with 5 box planes (1 face plane, 4 edge planes)
    let mut clip_points1: [ClipVertex; MAX_CONTACT_POINT] = Default::default();
    let mut clip_points2: [ClipVertex; MAX_CONTACT_POINT] = Default::default();

    // Clip to box side 1
    let np = clip_segment_to_line(
        &mut clip_points1,
        incident_edge,
        -side_normal,
        neg_side,
        neg_edge,
    );
    if np < 2 {
        return 0;
    }

    // Clip to negative box side 1
    let np = clip_segment_to_line(
        &mut clip_points2,
        clip_points1,
        side_normal,
        pos_side,
        pos_edge,
    );
    if np < 2 {
        return 0;
    }

    // Now clipPoints2 contains the clipping points.
    // Due to roundoff, it is possible that clipping removes all points.

    let mut num_contacts = 0_usize;
    for i in 0..MAX_CONTACT_POINT {
        let separation = dot(front_normal, clip_points2[i].v) - front;

        if separation <= 0.0 {
            contacts[num_contacts].separation = separation;
            contacts[num_contacts].normal = normal;
            // slide contact point onto reference face (easy to cull)
            contacts[num_contacts].position = clip_points2[i].v - (separation * front_normal);
            contacts[num_contacts].feature = clip_points2[i].fp;
            if axis == Axis::FaceBX || axis == Axis::FaceBY {
                flip(&mut contacts[num_contacts].feature)
            }
            num_contacts += 1;
        }
    }

    num_contacts
}
