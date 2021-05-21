use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use genmesh::generators::{Cube, IndexedPolygon};
use genmesh::{MapToVertices, Neighbors, Triangle, Triangulate, Vertices};

use mint::Vector3 as MintVector3;

//use cgmath::prelude::*;
use cgmath::{Decomposed, Transform};
use cgmath::{InnerSpace, Matrix3, Matrix4, Point2, Point3, Quaternion, SquareMatrix, Vector3};

use itertools::izip;

use std::collections::HashMap;
use std::ops::MulAssign;
use std::sync::Arc;

use crate::render::model::Model;
use crate::utils::{Normal, Vertex};

#[derive(Default, Debug, Clone)]
pub struct InterestingMeshData {
  pub map: HashMap<String, MyMeshData>,
}

#[derive(Debug, Clone)]
pub struct MyMesh {
  pub data: MyMeshData,
  print: bool,
  interesting: InterestingMeshData,
}

#[derive(Debug, Clone)]
pub struct MyMeshData {
  pub vertex: Vec<Point3<f32>>,
  pub tex: Vec<Point2<f32>>,
  pub tex_offset: Vec<Point2<i32>>,
  pub normals: Vec<Point3<f32>>,
  pub index: Vec<u32>,
  pub transform: Matrix4<f32>,
  pub inverse_transform: Matrix4<f32>,
}

impl MyMesh {
  pub fn new(
    vertex: Vec<cgmath::Point3<f32>>,
    tex: Vec<cgmath::Point2<f32>>,
    tex_offset: Vec<cgmath::Point2<i32>>,
    normals: Vec<cgmath::Point3<f32>>,
    index: Vec<u32>,
    transform: Matrix4<f32>,
    print: bool,
  ) -> MyMesh {
    MyMesh::new_interesting(
      vertex,
      tex,
      tex_offset,
      normals,
      index,
      transform,
      print,
      InterestingMeshData::default(),
    )
  }

  pub fn new_interesting(
    vertex: Vec<cgmath::Point3<f32>>,
    tex: Vec<cgmath::Point2<f32>>,
    tex_offset: Vec<cgmath::Point2<i32>>,
    normals: Vec<cgmath::Point3<f32>>,
    index: Vec<u32>,
    transform: Matrix4<f32>,
    print: bool,
    interesting: InterestingMeshData,
  ) -> MyMesh {
    let inverse_transform = transform.inverse_transform().unwrap();
    let data = MyMeshData {
      vertex,
      tex,
      tex_offset,
      normals,
      index,
      transform,
      inverse_transform,
    };
    let mesh = MyMesh {
      data,
      print,
      interesting: interesting,
    };
    if print {
      mesh.printstats();
    };
    mesh
  }

  pub fn reset_transform(&mut self) {
    self.data.transform = Matrix4::one();
  }

  pub fn get_buffers(&self, device: &Arc<Device>) -> Model {
    let vertices_vec: Vec<Vertex> = izip!(
      self.data.vertex.iter(),
      self.data.tex.iter(),
      self.data.tex_offset.iter()
    )
    .map(|(pos, tex, tex_offset)| (self.data.transform.transform_point(*pos), tex, tex_offset))
    .map(|(pos, tex, tex_offset)| Vertex {
      position: (pos[0], pos[1], pos[2]),
      tex: (tex.x, tex.y),
      tex_offset: (tex_offset.x, tex_offset.y),
    })
    .collect();
    let vertices = vertices_vec.iter().cloned();
    //println!("xxxxxxxxxxxxxxx vertices {:?}", vertices_vec);
    let normals_vec: Vec<Normal> = self
      .data
      .normals
      .iter()
      .map(|pos| self.data.transform.transform_point(*pos))
      .map(|pos| Normal {
        normal: (pos[0], pos[1], pos[2]),
      })
      .collect();
    let normals = normals_vec.iter().cloned();

    let indices = self.data.index.iter().cloned();

    let vertex_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices).unwrap();
    let index_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices).unwrap();

    let normals_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, normals).unwrap();
    Model::new(vertex_buffer, normals_buffer, index_buffer)
  }

  pub fn translation_decomposed(&self) -> (Vector3<f32>, Quaternion<f32>, [f32; 3]) {
    let m = &self.data.transform;
    let translation = Vector3::new(m[3][0], m[3][1], m[3][2]);
    let mut i = Matrix3::new(
      m[0][0], m[0][1], m[0][2], m[1][0], m[1][1], m[1][2], m[2][0], m[2][1], m[2][2],
    );
    let sx = i.x.magnitude();
    let sy = i.y.magnitude();
    let sz = i.determinant().signum() * i.z.magnitude();
    let scale = [sx, sy, sz];
    i.x.mul_assign(1.0 / sx);
    i.y.mul_assign(1.0 / sy);
    i.z.mul_assign(1.0 / sz);
    let r = _from_matrix(i);
    (translation, r, scale)
  }

  pub fn update_transform_2(
    &mut self,
    translation: Vector3<f32>,
    rotation: Matrix4<f32>,
    scale: [f32; 3],
  ) {
    let t = Matrix4::from_translation(translation);
    let s = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
    let tr = t * rotation * s;
    let inverse_transform = tr.inverse_transform().unwrap_or_else(|| {
      panic!("inverse failed for matrix {:?}", tr);
    });
    self.data.transform.concat_self(&tr);
    self.data.inverse_transform.concat_self(&inverse_transform);
  }

  pub fn printstats(&self) {
    let max_x = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::max);
    let min_x = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::min);
    let max_y = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::max);
    let min_y = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::min);
    let max_z = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.z)
      .fold(-0.0 / 0.0, f32::max);
    let min_z = self
      .data
      .vertex
      .iter()
      .cloned()
      .map(|p| p.z)
      .fold(-0.0 / 0.0, f32::min);
    println!(
      "mymesh: x ({}, {}) y ({}, {}) z ({}, {})",
      min_x, max_x, min_y, max_y, min_z, max_z
    );
    println!(
      "mesh properties: vertices {} normals {} indices {}",
      self.data.vertex.len(),
      self.data.normals.len(),
      self.data.index.len()
    );
    //println!("vertex {:?}", self.vertex);
    //println!("normal {:?}", self.normals);
  }

  pub fn add_bounding_box(&mut self, min: [f32; 3], max: [f32; 3]) {
    //println!("adding bounding box {:?} {:?}", min, max);
    let cube = Cube::new();
    let mut vertex: Vec<Point3<f32>> = cube
      .clone()
      .vertex(|v| Point3::<f32>::new(v.pos.x, v.pos.y, v.pos.z))
      .vertex(|v| {
        Point3::<f32>::new(
          if v.x < 0.0 { min[0] } else { max[0] },
          if v.y < 0.0 { min[1] } else { max[1] },
          if v.z < 0.0 { min[2] } else { max[2] },
        )
      })
      .vertices()
      .collect();

    let triangles: Vec<Triangle<usize>> = cube.indexed_polygon_iter().triangulate().collect();

    let neighbours = Neighbors::new(vertex.clone(), triangles.clone());

    let mut index: Vec<u32> = triangles
      .iter()
      .cloned()
      .vertices()
      .map(|v| v as u32)
      .collect();

    let mut normals: Vec<Point3<f32>> = (0..vertex.len())
      .map(|i| neighbours.normal_for_vertex(i, |v| MintVector3::<f32>::from([v.x, v.y, v.z])))
      .map(|v| Point3::from((v.x, v.y, v.z)))
      .collect();

    let mut tex = (0..vertex.len())
      .map(|_i| Point2::new(-1.0, -1.0))
      .collect();

    let mut tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();

    self.data.vertex.append(&mut vertex);
    self.data.normals.append(&mut normals);
    self.data.tex.append(&mut tex);
    self.data.tex_offset.append(&mut tex_offset);
    self.data.index.append(&mut index);
  }

  pub fn map_vertex<F>(&mut self, f: F)
  where
    F: Fn(&mut Point3<f32>),
  {
    for (_i, v) in self.data.vertex.iter_mut().enumerate() {
      f(&mut *v);
    }
    if self.print {
      self.printstats();
    }
  }

  pub fn add_consume(&mut self, other: &mut MyMesh) {
    self.data.add_consume(&mut other.data);
    self.interesting.add_consume(&mut other.interesting);
  }
}

impl MyMeshData {
  pub fn add_consume(&mut self, other: &mut MyMeshData) {
    let mult_mat = &other.transform;
    for vert in other.vertex.iter_mut() {
      *vert = mult_mat.transform_point(*vert);
    }
    for norm in other.normals.iter_mut() {
      *norm = mult_mat.transform_point(*norm);
    }

    let index_add: u32 = self.vertex.len() as u32;
    self.vertex.append(&mut other.vertex);
    self.normals.append(&mut other.normals);
    self.tex.append(&mut other.tex);
    self.tex_offset.append(&mut other.tex_offset);
    for ind in other.index.iter_mut() {
      *ind = index_add + *ind;
    }
    self.index.append(&mut other.index);
  }
}

impl InterestingMeshData {
  pub fn add_consume(&mut self, other: &mut InterestingMeshData) {
    for (key, val) in other.map.drain() {
      self.map.insert(key, val);
    }
  }
}

/// Convert a rotation matrix to an equivalent quaternion.
fn _from_matrix(mat: Matrix3<f32>) -> Quaternion<f32> {
  let trace = mat.trace();
  if trace >= 0.0 {
    let ss = (1.0 + trace).sqrt();
    let ww = 0.5 * ss;
    let ss = 0.5 / ss;
    let xx = (mat.y.z - mat.z.y) * ss;
    let yy = (mat.z.x - mat.x.z) * ss;
    let zz = (mat.x.y - mat.y.x) * ss;
    Quaternion::new(ww, xx, yy, zz)
  } else if (mat.x.x > mat.y.y) && (mat.x.x > mat.z.z) {
    let ss = ((mat.x.x - mat.y.y - mat.z.z) + 1.0).sqrt();
    let xx = 0.5 * ss;
    let ss = 0.5 / ss;
    let yy = (mat.y.x + mat.x.y) * ss;
    let zz = (mat.x.z + mat.z.x) * ss;
    let ww = (mat.y.z - mat.z.y) * ss;
    Quaternion::new(ww, xx, yy, zz)
  } else if mat.y.y > mat.z.z {
    let ss = ((mat.y.y - mat.x.x - mat.z.z) + 1.0).sqrt();
    let yy = 0.5 * ss;
    let ss = 0.5 / ss;
    let zz = (mat.z.y + mat.y.z) * ss;
    let xx = (mat.y.x + mat.x.y) * ss;
    let ww = (mat.z.x - mat.x.z) * ss;
    Quaternion::new(ww, xx, yy, zz)
  } else {
    let ss = ((mat.z.z - mat.x.x - mat.y.y) + 1.0).sqrt();
    let zz = 0.5 * ss;
    let ss = 0.5 / ss;
    let xx = (mat.x.z + mat.z.x) * ss;
    let yy = (mat.z.y + mat.y.z) * ss;
    let ww = (mat.x.y - mat.y.x) * ss;
    Quaternion::new(ww, xx, yy, zz)
  }
}

#[cfg(test)]
mod test {
  use crate::render::mymesh::MyMesh;
  use crate::things::primitives::PrimitiveCube;
  use cgmath::{Matrix4, One, Vector3};

  fn test_mesh() -> MyMesh {
    let mesh = PrimitiveCube::new(1.0, 1.0, 1.0, (1.0, 4.0, 9.0));
    mesh.mesh
  }

  #[test]
  pub fn test_identity_transform() {
    let mut cube = test_mesh();
    println!("cube {:?}", cube.data.transform);
    let (t, r, s) = cube.translation_decomposed();
    assert_eq!(t, Vector3::new(1.0, 4.0, 9.0));
    assert_eq!(s, [1.0, 1.0, 1.0]);
    cube.update_transform_2(Vector3::new(0.0, 0.0, 0.0), Matrix4::one(), [1.0, 1.0, 1.0]);
    // check nothing changed
    assert_eq!(t, Vector3::new(1.0, 4.0, 9.0));
    assert_eq!(s, [1.0, 1.0, 1.0]);
  }

  #[test]
  pub fn test_translate_transform() {
    let mut cube = test_mesh();
    println!("cube {:?}", cube.data.transform);
    cube.update_transform_2(Vector3::new(5.0, 6.0, 7.0), Matrix4::one(), [1.0, 1.0, 1.0]);
    // check nothing changed
    let (t, r, s) = cube.translation_decomposed();
    assert_eq!(t, Vector3::new(1.0 + 5.0, 4.0 + 6.0, 9.0 + 7.0));
    assert_eq!(s, [1.0, 1.0, 1.0]);
  }
}
