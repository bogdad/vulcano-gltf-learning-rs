use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use gltf::scene::Node;

//use cgmath::prelude::*;
use cgmath::Transform;
use cgmath::{InnerSpace, Matrix3, Matrix4, Point2, Point3, Quaternion, SquareMatrix, Vector3};

use itertools::izip;

use std::ops::MulAssign;
use std::path::Path;
use std::sync::Arc;

use crate::render::model::Model;
use crate::utils::{Normal, Vertex};

#[derive(Debug)]
pub struct MyMesh {
  pub vertex: Vec<Point3<f32>>,
  pub tex: Vec<Point2<f32>>,
  pub tex_offset: Vec<Point2<i32>>,
  pub normals: Vec<Point3<f32>>,
  pub index: Vec<u32>,
  pub transform: Matrix4<f32>,
}

impl MyMesh {
  pub fn new(
    vertex: Vec<cgmath::Point3<f32>>,
    tex: Vec<cgmath::Point2<f32>>,
    tex_offset: Vec<cgmath::Point2<i32>>,
    normals: Vec<cgmath::Point3<f32>>,
    index: Vec<u32>,
    transform: Matrix4<f32>,
  ) -> MyMesh {
    let max_x = vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::max);
    let min_x = vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::min);
    let max_y = vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::max);
    let min_y = vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::min);
    let max_z = vertex
      .iter()
      .cloned()
      .map(|p| p.z)
      .fold(-0.0 / 0.0, f32::max);
    let min_z = vertex
      .iter()
      .cloned()
      .map(|p| p.z)
      .fold(-0.0 / 0.0, f32::min);
    println!(
      "mymesh: x ({}, {}) y ({}, {}) z ({}, {})",
      min_x, max_x, min_y, max_y, min_z, max_z
    );
    MyMesh {
      vertex,
      tex,
      tex_offset,
      normals,
      index,
      transform,
    }
  }

  pub fn from_gltf(path: &Path) -> MyMesh {
    let (d, b, _i) = gltf::import(path).unwrap();
    let mesh = d.meshes().next().unwrap();
    let primitive = mesh.primitives().next().unwrap();
    let reader = primitive.reader(|buffer| Some(&b[buffer.index()]));
    let vertex = {
      let iter = reader.read_positions().unwrap_or_else(|| {
        panic!(
          "primitives must have the POSITION attribute (mesh: {}, primitive: {})",
          mesh.index(),
          primitive.index()
        )
      });

      iter
        .map(|arr| {
          //println!("p {:?}", arr);
          Point3::from(arr)
        })
        .collect::<Vec<_>>()
    };
    let tex = (0..vertex.len())
      .map(|_i| Point2::new(-1.0, -1.0))
      .collect();
    let tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();
    let normals = {
      let iter = reader.read_normals().unwrap_or_else(|| {
        panic!(
          "primitives must have the NORMALS attribute (mesh: {}, primitive: {})",
          mesh.index(),
          primitive.index()
        )
      });
      iter
        .map(|arr| {
          // println!("n {:?}", arr);
          Point3::from(arr)
        })
        .collect::<Vec<_>>()
    };
    let index = reader
      .read_indices()
      .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>());

    let node: Node = d.nodes().find(|node| node.mesh().is_some()).unwrap();
    let transform = Matrix4::from(node.transform().matrix());
    // let (translation, rotation, scale) = node.transform().decomposed();
    // println!("t {:?} r {:?} s {:?}", translation, rotation, scale);

    MyMesh::new(vertex, tex, tex_offset, normals, index.unwrap(), transform)
  }

  pub fn get_buffers(&self, device: &Arc<Device>) -> Model {
    let vertices_vec: Vec<Vertex> =
      izip!(self.vertex.iter(), self.tex.iter(), self.tex_offset.iter())
        .map(|(pos, tex, tex_offset)| (self.transform.transform_point(*pos), tex, tex_offset))
        .map(|(pos, tex, tex_offset)| Vertex {
          position: (pos[0], pos[1], pos[2]),
          tex: (tex.x, tex.y),
          tex_offset: (tex_offset.x, tex_offset.y),
        })
        .collect();
    let vertices = vertices_vec.iter().cloned();
    //println!("xxxxxxxxxxxxxxx vertices {:?}", vertices_vec);
    let normals_vec: Vec<Normal> = self
      .normals
      .iter()
      .map(|pos| self.transform.transform_point(*pos))
      .map(|pos| Normal {
        normal: (pos[0], pos[1], pos[2]),
      })
      .collect();
    let normals = normals_vec.iter().cloned();

    let indices = self.index.iter().cloned();

    println!(
      "mesh properties: vertices {} normals {} indices {}",
      vertices_vec.len(),
      normals_vec.len(),
      self.index.len()
    );

    let vertex_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices).unwrap();
    let index_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices).unwrap();

    let normals_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, normals).unwrap();
    Model::new(vertex_buffer, normals_buffer, index_buffer)
  }

  fn translation_decomposed(&self) -> (Vector3<f32>, Quaternion<f32>, [f32; 3]) {
    let m = &self.transform;
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
    let r = from_matrix(i);
    (translation, r, scale)
  }

  fn update_transform(
    &mut self,
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: [f32; 3],
  ) {
    let t = Matrix4::from_translation(translation);
    let r = Matrix4::from(rotation);
    let s = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
    self.transform = t * r * s;
  }

  pub fn update_transform_2(
    &mut self,
    translation: Vector3<f32>,
    rotation: Matrix4<f32>,
    scale: [f32; 3],
  ) {
    let t = Matrix4::from_translation(translation);
    let s = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
    self.transform = t * rotation * s;
  }
}

/// Convert a rotation matrix to an equivalent quaternion.
fn from_matrix(mat: Matrix3<f32>) -> Quaternion<f32> {
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
