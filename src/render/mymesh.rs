use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;

use genmesh::generators::{Cube, IndexedPolygon};
use genmesh::{MapToVertices, Neighbors, Triangle, Triangulate, Vertices};
use gltf::scene::Node;

use mint::Vector3 as MintVector3;

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
  print: bool,
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
    let mesh = MyMesh {
      vertex,
      tex,
      tex_offset,
      normals,
      index,
      transform,
      print,
    };
    if print {
      mesh.printstats();
    };
    mesh
  }

  pub fn from_gltf(path: &Path, print: bool) -> MyMesh {
    let (d, b, _i) = gltf::import(path).unwrap();
    let mesh = d.meshes().next().unwrap();
    let mut all_vertex = vec![];
    let mut all_normals = vec![];
    let mut all_tex = vec![];
    let mut all_tex_offset = vec![];
    let mut all_index = vec![];
    let mut bounding_boxes = vec![];
    let mut last_index = 0;
    for primitive in mesh.primitives() {
      println!("- Primitive #{}", primitive.index());
      for (semantic, _) in primitive.attributes() {
        println!("-- {:?}", semantic);
      }
      println!("{:?}", primitive.bounding_box());
      bounding_boxes.push(primitive.bounding_box().clone());
      let reader = primitive.reader(|buffer| Some(&b[buffer.index()]));
      let mut vertex = {
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
      let mut tex = (0..vertex.len())
        .map(|_i| Point2::new(-1.0, -1.0))
        .collect();

      let mut tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();
      let mut normals = {
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

      all_normals.append(&mut normals);
      all_tex.append(&mut tex);
      all_tex_offset.append(&mut tex_offset);
      let mut read_index = index.unwrap();
      for ind in read_index.iter_mut() {
        *ind = last_index + *ind;
      }
      all_index.append(&mut read_index);
      last_index += vertex.len() as u32;
      all_vertex.append(&mut vertex);
    }
    let node: Node = d.nodes().find(|node| node.mesh().is_some()).unwrap();
    let transform = Matrix4::from(node.transform().matrix());
    // let (translation, rotation, scale) = node.transform().decomposed();
    // println!("t {:?} r {:?} s {:?}", translation, rotation, scale);
    let mut res = MyMesh::new(
      all_vertex,
      all_tex,
      all_tex_offset,
      all_normals,
      all_index,
      transform,
      print,
    );
    if print {
      for bounding_box in bounding_boxes {
        res.add_bounding_box(bounding_box.min, bounding_box.max);
      }
    }
    res
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

    /*println!(
      "mesh properties: vertices {} normals {} indices {}",
      vertices_vec.len(),
      normals_vec.len(),
      self.index.len()
    );*/

    let vertex_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices).unwrap();
    let index_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices).unwrap();

    let normals_buffer =
      CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, normals).unwrap();
    Model::new(vertex_buffer, normals_buffer, index_buffer)
  }

  fn _translation_decomposed(&self) -> (Vector3<f32>, Quaternion<f32>, [f32; 3]) {
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
    let r = _from_matrix(i);
    (translation, r, scale)
  }

  fn _update_transform(
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

  pub fn printstats(&self) {
    let max_x = self
      .vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::max);
    let min_x = self
      .vertex
      .iter()
      .cloned()
      .map(|p| p.x)
      .fold(-0.0 / 0.0, f32::min);
    let max_y = self
      .vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::max);
    let min_y = self
      .vertex
      .iter()
      .cloned()
      .map(|p| p.y)
      .fold(-0.0 / 0.0, f32::min);
    let max_z = self
      .vertex
      .iter()
      .cloned()
      .map(|p| p.z)
      .fold(-0.0 / 0.0, f32::max);
    let min_z = self
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
      self.vertex.len(),
      self.normals.len(),
      self.index.len()
    );
    //println!("vertex {:?}", self.vertex);
    //println!("normal {:?}", self.normals);
  }

  pub fn add_bounding_box(&mut self, min: [f32; 3], max: [f32; 3]) {
    println!("adding bounding box {:?} {:?}", min, max);
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

    self.vertex.append(&mut vertex);
    self.normals.append(&mut normals);
    self.tex.append(&mut tex);
    self.tex_offset.append(&mut tex_offset);
    self.index.append(&mut index);
  }

  pub fn map_vertex<F>(&mut self, f: F)
  where
    F: Fn(&mut Point3<f32>),
  {
    for (_i, v) in self.vertex.iter_mut().enumerate() {
      f(&mut *v);
    }
    if self.print {
      self.printstats();
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
