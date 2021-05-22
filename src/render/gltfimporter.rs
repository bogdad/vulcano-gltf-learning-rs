use cgmath::{Matrix4, Point2, Point3, Transform};
use gltf::buffer;
use gltf::mesh::BoundingBox;
use gltf::scene::Node;

use std::collections::HashMap;
use std::option::Option;
use std::path::Path;

use crate::render::mymesh::{InterestingMeshData, MyMesh, MyMeshData};

type Vertex = Vec<Point3<f32>>;
type Normals = Vec<Point3<f32>>;
type Tex = Vec<Point2<f32>>;
type TexOffset = Vec<Point2<i32>>;
type Index = Vec<u32>;
type Trans = Matrix4<f32>;
type InvTrans = Matrix4<f32>;

#[derive(Default, Debug)]
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
    current_transform: Option<(Trans, InvTrans)>,
  ) {
    if let Some((current_transform, _)) = current_transform {
      for vert in vertex.iter_mut() {
        *vert = current_transform.transform_point(*vert);
      }
      for norm in normals.iter_mut() {
        *norm = current_transform.transform_point(*norm);
      }
    }
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

#[derive(Debug)]
struct VisitState {
  b: Vec<buffer::Data>,
  state: State,
  interesting_state: State,
  bounding_boxes: Vec<BoundingBox>,
  print: bool,
  interesting_map: HashMap<String, MyMeshData>,
}

impl VisitState {
  pub fn finish(self, transform: Trans, print: bool) -> MyMesh {
    let mut res = self.state.build_mesh(self.interesting_map, transform, print);

    if print {
      for bounding_box in self.bounding_boxes {
        res.add_bounding_box(bounding_box.min, bounding_box.max);
      }
    }
    res
  }
}

fn collect_mesh(
  visit_state: &mut VisitState,
  node: &Node,
  parent_transforms: Option<(Trans, InvTrans)>,
) {
  let current_transform = if let Some((parent_transform, parent_inv_transform)) = parent_transforms {
    let mut transform = Matrix4::from(node.transform().matrix());
    let mut inverse_transform = transform.inverse_transform().unwrap();
    transform = parent_transform.concat(&transform);
    inverse_transform = parent_inv_transform.concat(&inverse_transform);
    Some((transform, inverse_transform))
  } else {
    None
  };

  if let Some(mesh) = node.mesh() {
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
      visit_state
        .bounding_boxes
        .push(primitive.bounding_box().clone());
      let reader = primitive.reader(|buffer| Some(&visit_state.b[buffer.index()]));
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
          current_transform,
        );
      }
      if visit_state.print {
        println!(
          "- mesh Primitive {:?} #{} v {:?} i {:?}",
          mesh.name(),
          primitive.index(),
          vertex.len(),
          index.len()
        );
      }
      visit_state.state.collect(
        &mut vertex,
        &mut normals,
        &mut tex,
        &mut tex_offset,
        &mut index,
        current_transform,
      );
    }
    if let Some(interesting_name) = interesting_name {
      let (transform, inverse_transform) = if let Some((transform, inverse_transform)) = current_transform {
        (transform, inverse_transform)
      } else {
        (Matrix4::one(), Matrix4::one())
      };
      let interesting_mesh_data = interesting_state.build_mesh_data(transform, inverse_transform);
      if visit_state.print {
        println!(
          "part {:?} vertices {:?} indices {:?}",
          interesting_name.to_string(),
          interesting_mesh_data.vertex.len(),
          interesting_mesh_data.index.len()
        );
      }
      visit_state
        .interesting_map
        .insert(interesting_name.to_string(), interesting_mesh_data);
    }
  }
  /*
     root:
     current: none  ->  matrix one.Matrix4
     non root:
     current: prev node transfartm -> prev node transform * this node transform.
  */
  let next_transform = if let Some((current_transform, current_inverse)) = current_transform {
    Some((current_transform, current_inverse))
  } else {
    Some((Matrix4::one(), Matrix4::one()))
  };
  for child_node in node.children() {
    collect_mesh(visit_state, &child_node, next_transform);
  }
}

pub fn from_gltf(path: &Path, print: bool) -> MyMesh {
  println!("glb {:?}", path);
  let (d, b, _i) = gltf::import(path).unwrap();

  let default_scene = d.default_scene().unwrap();
  if default_scene.nodes().len() != 1 {
    panic!("expect default scene to have one root node");
  }
  let root_node = default_scene.nodes().next().unwrap();

  let mut visit_state = VisitState {
    b,
    state: State::default(),
    interesting_state: State::default(),
    bounding_boxes: Vec::default(),
    interesting_map: HashMap::default(),
    print: print,
  };
  collect_mesh(&mut visit_state, &root_node, None);
  let transform = Matrix4::from(root_node.transform().matrix());
  visit_state.finish(transform, print)
}
