use cgmath::{Matrix4, Point2, Point3, Transform};
use gltf::scene::Node;

use std::collections::HashMap;
use std::path::Path;

use crate::render::mymesh::{InterestingMeshData, MyMesh, MyMeshData};

type Vertex = Vec<Point3<f32>>;
type Normals = Vec<Point3<f32>>;
type Tex = Vec<Point2<f32>>;
type TexOffset = Vec<Point2<i32>>;
type Index = Vec<u32>;
type Trans = Matrix4<f32>;

#[derive(Default)]
struct State {
  last_index: u32,
  all_vertex: Vertex,
  all_normals: Normals,
  all_tex: Tex,
  all_tex_offset: TexOffset,
  all_index: Index,
}

impl State {
  pub fn collect(
    &mut self,
    vertex: &mut Vertex,
    normals: &mut Normals,
    tex: &mut Tex,
    tex_offset: &mut TexOffset,
    index: &mut Index,
  ) {
    self.all_normals.append(normals);
    self.all_tex.append(tex);
    self.all_tex_offset.append(tex_offset);
    for ind in index.iter_mut() {
      *ind = self.last_index + *ind;
    }
    self.all_index.append(index);
    self.last_index += vertex.len() as u32;
    self.all_vertex.append(vertex);
  }

  pub fn build_mesh_data(self, trans: Trans, inv_trans: Trans) -> MyMeshData {
    MyMeshData {
      vertex: self.all_vertex,
      normals: self.all_normals,
      tex: self.all_tex,
      tex_offset: self.all_tex_offset,
      index: self.all_index,
      transform: trans,
      inverse_transform: inv_trans,
    }
  }

  pub fn build_mesh(
    self,
    interesting_map: HashMap<String, MyMeshData>,
    transform: Trans,
    print: bool,
  ) -> MyMesh {
    let interesting = InterestingMeshData {
      map: interesting_map,
    };
    MyMesh::new_interesting(
      self.all_vertex,
      self.all_tex,
      self.all_tex_offset,
      self.all_normals,
      self.all_index,
      transform,
      print,
      interesting,
    )
  }
}

pub fn from_gltf(path: &Path, print: bool) -> MyMesh {
  let (d, b, _i) = gltf::import(path).unwrap();
  let mut state = State::default();
  let mut bounding_boxes = vec![];
  let mut interesting_map: HashMap<String, MyMeshData> = HashMap::new();
  let node: Node = d.nodes().find(|node| node.mesh().is_some()).unwrap();
  let transform = Matrix4::from(node.transform().matrix());
  let inverse_transform = transform.inverse_transform().unwrap();
  println!("glb {:?}", path);
  for mesh in d.meshes() {
    let name_opt = mesh.name();
    let interesting_name = name_opt.and_then(|name| {
      if name.starts_with("interesting") {
        let split: Vec<&str> = name.split("_").collect();
        if split.len() > 1 {
          Some(split[1])
        } else {
          None
        }
      } else {
        None
      }
    });
    let mut interesting_state = State::default();
    for primitive in mesh.primitives() {
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
      let mut tex: Vec<Point2<f32>> = (0..vertex.len())
        .map(|_i| Point2::new(-1.0, -1.0))
        .collect();

      let mut tex_offset: Vec<Point2<i32>> =
        (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();
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
      let mut index = reader
        .read_indices()
        .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
        .unwrap();
      if interesting_name.is_some() {
        interesting_state.collect(
          &mut vertex.clone(),
          &mut normals.clone(),
          &mut tex.clone(),
          &mut tex_offset.clone(),
          &mut index.clone(),
        );
      }
      if print {
        println!(
          "- mesh Primitive {:?} #{} v {:?} i {:?}",
          mesh.name(),
          primitive.index(),
          vertex.len(),
          index.len()
        );
      }
      state.collect(
        &mut vertex,
        &mut normals,
        &mut tex,
        &mut tex_offset,
        &mut index,
      );
    }
    if let Some(interesting_name) = interesting_name {
      let interesting_mesh_data = interesting_state.build_mesh_data(transform, inverse_transform);
      if print {
        println!(
          "part {:?} vertices {:?} indices {:?}",
          interesting_name.to_string(),
          interesting_mesh_data.vertex.len(),
          interesting_mesh_data.index.len()
        );
      }
      interesting_map.insert(interesting_name.to_string(), interesting_mesh_data);
    }
  }
  // let (translation, rotation, scale) = node.transform().decomposed();
  // println!("t {:?} r {:?} s {:?}", translation, rotation, scale);
  let mut res = state.build_mesh(interesting_map, transform, print);

  if print {
    for bounding_box in bounding_boxes {
      res.add_bounding_box(bounding_box.min, bounding_box.max);
    }
  }
  res
}
