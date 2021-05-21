use cgmath::{Matrix4, Point2, Point3, Transform};
use gltf::scene::Node;

use std::collections::HashMap;
use std::path::Path;

use crate::render::mymesh::{InterestingMeshData, MyMesh, MyMeshData};

pub fn from_gltf(path: &Path, print: bool) -> MyMesh {
  let (d, b, _i) = gltf::import(path).unwrap();
  let mut all_vertex = vec![];
  let mut all_normals = vec![];
  let mut all_tex = vec![];
  let mut all_tex_offset = vec![];
  let mut all_index = vec![];
  let mut bounding_boxes = vec![];
  let mut last_index = 0;
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
    let mut interesting_vertex = vec![];
    let mut interesting_normals = vec![];
    let mut interesting_tex = vec![];
    let mut interesting_tex_offset = vec![];
    let mut interesting_index = vec![];
    let mut last_interesting_index = 0;
    for primitive in mesh.primitives() {
      //for (semantic, _) in primitive.attributes() {
      //  println!("-- {:?}", semantic);
      //}
      //println!("{:?}", primitive.bounding_box());
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
        interesting_vertex.append(&mut vertex.clone());
        interesting_normals.append(&mut normals.clone());
        interesting_tex.append(&mut tex.clone());
        interesting_tex_offset.append(&mut tex_offset.clone());
        let mut index_clone = index.clone();
        for ind in index_clone.iter_mut() {
          *ind = last_interesting_index + *ind;
        }
        interesting_index.append(&mut index_clone);
        last_interesting_index += vertex.len() as u32;
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
      all_normals.append(&mut normals);
      all_tex.append(&mut tex);
      all_tex_offset.append(&mut tex_offset);
      for ind in index.iter_mut() {
        *ind = last_index + *ind;
      }
      all_index.append(&mut index);
      last_index += vertex.len() as u32;
      all_vertex.append(&mut vertex);
    }
    if let Some(interesting_name) = interesting_name {
      let interesting_mesh_data = MyMeshData {
        vertex: interesting_vertex,
        normals: interesting_normals,
        tex: interesting_tex,
        tex_offset: interesting_tex_offset,
        index: interesting_index,
        transform,
        inverse_transform,
      };
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
  let interesting = InterestingMeshData {
    map: interesting_map,
  };
  let mut res = MyMesh::new_interesting(
    all_vertex,
    all_tex,
    all_tex_offset,
    all_normals,
    all_index,
    transform,
    print,
    interesting,
  );
  if print {
    for bounding_box in bounding_boxes {
      res.add_bounding_box(bounding_box.min, bounding_box.max);
    }
  }
  res
}
