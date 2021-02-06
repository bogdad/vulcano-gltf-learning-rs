use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;
use vulkano::device::Device;
use vulkano::pipeline::GraphicsPipelineAbstract;

use gltf::scene::Node;

//use cgmath::prelude::*;
use cgmath::Transform;
use cgmath::{InnerSpace, Matrix4, Matrix3, Point3, SquareMatrix, Quaternion, Vector3};

use std::path::Path;
use std::sync::Arc;
use std::ops::MulAssign;
use std::fmt;

use crate::utils::{Normal, Vertex};

#[derive(Debug)]
pub struct MyMesh {
  pub vertex: Vec<Point3<f32>>,
  pub normals: Vec<Point3<f32>>,
  pub index: Vec<u32>,
  pub transform: Matrix4<f32>,
}

impl MyMesh {
  pub fn new(
    vertex: Vec<cgmath::Point3<f32>>,
    normals: Vec<cgmath::Point3<f32>>,
    index: Vec<u32>,
    transform: Matrix4<f32>,
  ) -> MyMesh {
    MyMesh {
      vertex,
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

    MyMesh::new(vertex, normals, index.unwrap(), transform)
  }

  pub fn get_buffers(&self, device: &Arc<Device>) -> Model {
    let vertices_vec: Vec<Vertex> = self
      .vertex
      .iter()
      .map(|pos| self.transform.transform_point(*pos))
      .map(|pos| Vertex {
        position: (pos[0], pos[1], pos[2]),
      })
      .collect();
    let vertices = vertices_vec.iter().cloned();
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
        m[0][0], m[0][1], m[0][2],
        m[1][0], m[1][1], m[1][2],
        m[2][0], m[2][1], m[2][2],
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

  fn update_transform(&mut self, translation: Vector3<f32>, rotation: Quaternion<f32>, scale: [f32; 3]) {
    let t = Matrix4::from_translation(translation);
    let r = Matrix4::from(rotation);
    let s = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
    self.transform = t * r * s;
  }

  pub fn update_transform_2(&mut self, translation: Vector3<f32>, rotation: Matrix4<f32>, scale: [f32; 3]) {
    let t = Matrix4::from_translation(translation);
    let s = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
    self.transform = t * rotation * s;
  }
}

#[derive(Clone, Debug)]
pub struct Model {
  vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
  normals: Arc<CpuAccessibleBuffer<[Normal]>>,
  index: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Model {
  pub fn new(
    vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
    normals: Arc<CpuAccessibleBuffer<[Normal]>>,
    index: Arc<CpuAccessibleBuffer<[u32]>>,
  ) -> Model {
    Model {
      vertex,
      normals,
      index,
    }
  }

  pub fn draw_indexed<S>(
    &self,
    builder: &mut AutoCommandBufferBuilder<StandardCommandPoolBuilder>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    set: S,
  ) where
    S: DescriptorSetsCollection,
  {
    builder
      .draw_indexed(
        pipeline.clone(),
        &DynamicState::none(),
        vec![self.vertex.clone(), self.normals.clone()],
        self.index.clone(),
        set,
        (),
      )
      .unwrap();
  }

  pub fn from_gltf(path: &Path, device: &Arc<Device>) -> Model {
    MyMesh::from_gltf(path).get_buffers(device)
  }
}

/// Convert a rotation matrix to an equivalent quaternion.
fn from_matrix(m: Matrix3<f32>) -> Quaternion<f32> {
  let trace = m.trace();
  if trace >= 0.0 {
      let s = (1.0 + trace).sqrt();
      let w = 0.5 * s;
      let s = 0.5 / s;
      let x = (m.y.z - m.z.y) * s;
      let y = (m.z.x - m.x.z) * s;
      let z = (m.x.y - m.y.x) * s;
      Quaternion::new(w, x, y, z)
  } else if (m.x.x > m.y.y) && (m.x.x > m.z.z) {
      let s = ((m.x.x - m.y.y - m.z.z) + 1.0).sqrt();
      let x = 0.5 * s;
      let s = 0.5 / s;
      let y = (m.y.x + m.x.y) * s;
      let z = (m.x.z + m.z.x) * s;
      let w = (m.y.z - m.z.y) * s;
      Quaternion::new(w, x, y, z)
  } else if m.y.y > m.z.z {
      let s = ((m.y.y - m.x.x - m.z.z) + 1.0).sqrt();
      let y = 0.5 * s;
      let s = 0.5 / s;
      let z = (m.z.y + m.y.z) * s;
      let x = (m.y.x + m.x.y) * s;
      let w = (m.z.x - m.x.z) * s;
      Quaternion::new(w, x, y, z)
  } else {
      let s = ((m.z.z - m.x.x - m.y.y) + 1.0).sqrt();
      let z = 0.5 * s;
      let s = 0.5 / s;
      let x = (m.x.z + m.z.x) * s;
      let y = (m.z.y + m.y.z) * s;
      let w = (m.x.y - m.y.x) * s;
      Quaternion::new(w, x, y, z)
  }
}
