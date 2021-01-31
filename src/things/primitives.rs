use genmesh::generators::{Cube, IndexedPolygon};
use genmesh::{Triangulate, MapToVertices, Vertices, Neighbors, Triangle};
use cgmath::{Matrix4, One, Point3, Vector3};
use mint::Vector3 as MintVector3;

use crate::render::MyMesh;

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

    let triangles: Vec<Triangle<usize>> = cube
      .indexed_polygon_iter()
      .triangulate()
      .collect();

    let neighbours = Neighbors::new(vertex.clone(), triangles.clone());

    let index: Vec<u32> =
      triangles
      .iter()
      .cloned()
      .vertices()
      .map(|v| v as u32)
      .collect();

    let normals: Vec<Point3<f32>> = (0..vertex.len())
      .map(|i| neighbours.normal_for_vertex(i, |v|{ MintVector3::<f32>::from([v.x, v.y, v.z]) }))
      .map(|v| Point3::from((v.x, v.y, v.z)))
      .collect();

    let transform = Matrix4::one();

    let mut mesh = MyMesh::new(vertex, normals, index, transform);
    mesh.update_transform_2(Vector3::from(xx), Matrix4::one(), [x, y, z]);
    println!("mesh {:?}", mesh);
    PrimitiveCube {
      mesh: mesh
    }
  }
}
