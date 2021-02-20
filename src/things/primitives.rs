use cgmath::{EuclideanSpace, Matrix4, One, Point2, Point3, Vector3};
use genmesh::generators::{Cube, IndexedPolygon};
use genmesh::{MapToVertices, Neighbors, Triangle, Triangulate, Vertices};
use mint::Vector3 as MintVector3;

use crate::render::mymesh::MyMesh;

pub struct PrimitiveCube {
  pub mesh: MyMesh,
}

impl PrimitiveCube {
  pub fn new(x: f32, y: f32, z: f32, xx: (f32, f32, f32)) -> PrimitiveCube {
    let cube = Cube::new();
    let vertex: Vec<Point3<f32>> = cube
      .clone()
      .vertex(|v| Point3::<f32>::new(v.pos.x, v.pos.y, v.pos.z))
      .vertices()
      .collect();

    let triangles: Vec<Triangle<usize>> = cube.indexed_polygon_iter().triangulate().collect();

    let neighbours = Neighbors::new(vertex.clone(), triangles.clone());

    let index: Vec<u32> = triangles
      .iter()
      .cloned()
      .vertices()
      .map(|v| v as u32)
      .collect();

    let normals: Vec<Point3<f32>> = (0..vertex.len())
      .map(|i| neighbours.normal_for_vertex(i, |v| MintVector3::<f32>::from([v.x, v.y, v.z])))
      .map(|v| Point3::from((v.x, v.y, v.z)))
      .collect();

    let transform = Matrix4::one();

    let tex = (0..vertex.len())
      .map(|_i| Point2::new(-1.0, -1.0))
      .collect();

    let tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();

    let mut mesh = MyMesh::new(vertex, tex, tex_offset, normals, index, transform);
    mesh.update_transform_2(Vector3::from(xx), Matrix4::one(), [x, y, z]);
    println!("mesh {:?}", mesh);
    PrimitiveCube { mesh }
  }
}

pub struct PrimitiveTriangle {
  pub mesh: MyMesh,
}

impl PrimitiveTriangle {
  pub fn new(pos: Point3<f32>) -> Self {
    let vertex: Vec<Point3<f32>> = vec![
      Point3::new(0.0, 0.0, 0.0),
      Point3::new(0.0, -1.0, 0.0),
      Point3::new(1.0, 0.0, 0.0),
    ];

    let index: Vec<u32> = vec![0, 1, 2];

    let normals: Vec<Point3<f32>> = vec![
      Point3::new(0.0, 0.0, -1.0),
      Point3::new(0.0, 0.0, -1.0),
      Point3::new(0.0, 0.0, -1.0),
    ];

    let transform = Matrix4::one();
    let tex = (0..vertex.len())
      .map(|_i| Point2::new(-1.0, -1.0))
      .collect();
    let tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();
    let mut mesh = MyMesh::new(vertex, tex, tex_offset, normals, index, transform);
    mesh.update_transform_2(pos.to_vec(), Matrix4::one(), [10.0, 10.0, 10.0]);
    println!("mesh {:?}", mesh);
    PrimitiveTriangle { mesh }
  }

  pub fn new_tex(
    pos: Point3<f32>,
    tex_pos_min: Point2<f32>,
    tex_pos_max: Point2<f32>,
    texture_size: (u32, u32),
  ) -> Self {
    let vertex: Vec<Point3<f32>> = vec![
      pos + Vector3::new(0.0, 0.0, 0.0),
      pos + Vector3::new(0.0, -1.0, 0.0),
      pos + Vector3::new(1.0, 0.0, 0.0),
    ];
    //println!("new tex min {:?} max {:?}", tex_pos_min, tex_pos_max);
    let tex = vec![
      Point2::new(
        tex_pos_min.x / (texture_size.0 as f32),
        tex_pos_max.y / (texture_size.1 as f32),
      ),
      Point2::new(
        tex_pos_min.x / (texture_size.0 as f32),
        tex_pos_min.y / (texture_size.1 as f32),
      ),
      Point2::new(
        tex_pos_max.x / (texture_size.0 as f32),
        tex_pos_max.y / (texture_size.1 as f32),
      ),
    ];

    let tex_offset = vec![Point2::new(0, 0), Point2::new(0, 0), Point2::new(0, 0)];

    //println!("tex: {:?}", tex);

    let index: Vec<u32> = vec![0, 1, 2];

    let normals: Vec<Point3<f32>> = vec![
      Point3::new(0.0, 0.0, -1.0),
      Point3::new(0.0, 0.0, -1.0),
      Point3::new(0.0, 0.0, -1.0),
    ];

    let transform = Matrix4::one();

    let mesh = MyMesh::new(vertex, tex, tex_offset, normals, index, transform);
    //mesh.update_transform_2(pos.to_vec(), Matrix4::one(), [10.0, 10.0, 10.0]);
    println!("mesh {:?}", mesh);
    println!("tex: {:?}", mesh.tex);
    PrimitiveTriangle { mesh }
  }
}
