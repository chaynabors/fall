use std::collections::VecDeque;

use half_edge_mesh::FaceRc;
use half_edge_mesh::HalfEdgeMesh;
use nalgebra::Point3;
use nalgebra::Vector3;

fn line_to_pt_dist_sq(pt1: Point3<f32>, pt2: Point3<f32>, target: Point3<f32>) -> f32 {
    let line = pt2 - pt1;
    let p1_to_target = target - pt1;
    let p2_to_target = target - pt2;
    let p1_t_on_line = p1_to_target.dot(&line);

    if p1_t_on_line < 0.0 { return p1_to_target.magnitude_squared(); }
    let line_length2 = line.magnitude_squared();
    if p1_t_on_line >= line_length2 { return p2_to_target.magnitude_squared(); }
    p1_to_target.magnitude_squared() - p1_t_on_line * p1_t_on_line / line_length2
}

fn triangle_normal(pt1: Point3<f32>, pt2: Point3<f32>, pt3: Point3<f32>) -> Vector3<f32> {
    (pt2 - pt1).cross(&(pt3 - pt1)).normalize()
}

fn triangle_center(pt1: Point3<f32>, pt2: Point3<f32>, pt3: Point3<f32>) -> Point3<f32> {
    Point3::new((pt1.to_homogeneous() + pt2.to_homogeneous() + pt3.to_homogeneous()) / 3.0)
}

#[derive(Copy, Clone, Debug)]
struct Pair {
    pub idx: usize,
    pub pt: Point3<f32>,
}

impl Pair {
  fn new(i: usize, p: Point3<f32>) -> Pair { Pair { idx: i, pt: p } }
}

fn update_min_max(idx_pair: Pair, start: usize, pairs: &mut [Pair; 6]) {
  let mut replaced = false;

  match start {
    0 => {
      if idx_pair.pt.x < pairs[0].pt.x {
        update_min_max(pairs[0], 1, pairs);
        pairs[0] = idx_pair;
        replaced = true;
      }
    },
    1 => {
      if idx_pair.pt.x > pairs[1].pt.x {
        update_min_max(pairs[1], 2, pairs);
        pairs[1] = idx_pair;
        replaced = true;
      }
    },
    2 => {
      if idx_pair.pt.y < pairs[2].pt.y {
        update_min_max(pairs[2], 3, pairs);
        pairs[2] = idx_pair;
        replaced = true;
      }
    },
    3 => {
      if idx_pair.pt.y > pairs[3].pt.y {
        update_min_max(pairs[3], 4, pairs);
        pairs[3] = idx_pair;
        replaced = true;
      }
    },
    4 => {
      if idx_pair.pt.z < pairs[4].pt.z {
        update_min_max(pairs[4], 5, pairs);
        pairs[4] = idx_pair;
        replaced = true;
      }
    },
    5 => {
      if idx_pair.pt.z > pairs[5].pt.z {
        pairs[5] = idx_pair;
        replaced = true;
      }
    },
    _ => { return; },
  }

  if !replaced { update_min_max(idx_pair, start + 1, pairs); }
}

fn construct_tetrahedron_order(p0: Pair, p1: Pair, p2: Pair, p3: Pair) -> Vec<usize> {
    match (p0.pt - p3.pt).dot(&triangle_normal(p0.pt, p1.pt, p2.pt)) < 0.0 {
        true => vec![p3.idx, p0.idx, p1.idx, p2.idx],
        false => vec![p3.idx, p1.idx, p0.idx, p2.idx],
    }
}

fn get_extreme_points(list: & Vec<Point3<f32>>) -> Vec<usize> {
  debug_assert!(list.len() >= 4);

  let mut boundaries = [Pair::new(0, list[0]); 6];

  for (i, pt) in list.iter().cloned().enumerate() {
    update_min_max(Pair::new(i, pt), 0, &mut boundaries);
  }

  let mut p0 = boundaries[0];
  let mut p1 = boundaries[1];
  let mut pt_dist_sq_max = (p0.pt - p1.pt).magnitude2();
  for (idx_a, pair_a) in boundaries.iter().enumerate() {
    for pair_b in boundaries[(idx_a + 1)..].iter() {
      let dist = (pair_a.pt - pair_b.pt).magnitude2();
      if dist > pt_dist_sq_max {
        pt_dist_sq_max = dist;
        p0 = pair_a.clone();
        p1 = pair_b.clone();
      }
    }
  }

  let mut p2 = boundaries[0];
  let mut line_dist_sq_max = 0.0;
  for pair in boundaries.iter() {
    if pair.idx == p0.idx || pair.idx == p1.idx { continue; }
    let dist = line_to_pt_dist_sq(p0.pt, p1.pt, pair.pt);
    if dist > line_dist_sq_max {
      line_dist_sq_max = dist;
      p2 = pair.clone();
    }
  }

  let mut p3 = boundaries[0];
  let mut tri_dist_sq_max = 0.0;
  let face_center = triangle_center(p0.pt, p1.pt, p2.pt);
  for pair in boundaries.iter() {
    if pair.idx == p0.idx || pair.idx == p1.idx || pair.idx == p2.idx { continue; }
    let dist = (pair.pt - face_center).magnitude2();
    if dist > tri_dist_sq_max {
      tri_dist_sq_max = dist;
      p3 = pair.clone();
    }
  }

  return construct_tetrahedron_order(p0, p1, p2, p3);
}

pub fn build_convex_hull(mut points_list: Vec<Point3<f32>>) -> HalfEdgeMesh {
    // Check that we have a valid list of points
    if points_list.len() < 4 { return HalfEdgeMesh::empty(); }
    let mut tet_points = get_extreme_points(& points_list);
    let mut hull_mesh = HalfEdgeMesh::from_tetrahedron_pts(points_list[tet_points[0]], points_list[tet_points[1]], points_list[tet_points[2]], points_list[tet_points[3]]);

    tet_points.sort_by(|a, b| a.cmp(b).reverse());
    tet_points.dedup();
    for i in tet_points.into_iter() {
        if i < points_list.len() { points_list.remove(i); }
    }

    points_list.retain(|p| {
        hull_mesh.faces.values().any(|f| f.borrow().can_see(& p))
    });

    let mut face_queue: VecDeque<FaceRc> = hull_mesh.faces.values().cloned().collect();

    while let Some(test_face) = face_queue.pop_front() {
        if !hull_mesh.faces.contains_key(& test_face.borrow().id) { continue; }

        let face_visible_points: Vec<Point3<f32>> = points_list.iter()
            .filter(|pt| test_face.borrow().can_see(pt))
            .cloned()
            .collect();

        let (point_maxima, _) = face_visible_points.iter()
            .enumerate()
            .fold((None, 0.0), |(mut point_maxima, mut max_dist), (idx, pt)| {
              let dist = test_face.borrow().directed_distance_to(pt);
              if dist > max_dist {
                point_maxima = Some((idx, pt.clone()));
                max_dist = dist;
              }
              (point_maxima, max_dist)
            });

        if point_maxima.is_none() { continue; }
        let (max_index, max_point) = point_maxima.unwrap();

        points_list.remove(max_index);

        let light_faces: Vec<FaceRc> = hull_mesh.faces.values().filter(|f| f.borrow().can_see(& max_point)).cloned().collect();

        match hull_mesh.attach_point_for_faces(max_point, & light_faces) {
            Ok(new_faces) => {
                points_list.retain(|p| {
                    face_visible_points.iter().all(|face_pt| *face_pt != *p) || new_faces.iter().any(|n_face| n_face.borrow().can_see(p))
                });

                face_queue.extend(new_faces);
            },
            Err(message) => { println!("Error occurred while attaching a new point, {}", message); },
        }
    }

    return hull_mesh;
}
