use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::pipeline::{GraphicsPipelineAbstract};
use vulkano::descriptor::descriptor_set::DescriptorSetsCollection;

use gltf::scene::Node;

use cgmath::Transform;
use cgmath::{Matrix4, Point3};

use std::sync::Arc;
use std::path::Path;

use crate::utils::{Vertex, Normal};


pub struct MyMesh {
    pub vertex: Vec<Point3<f32>>,
    pub normal: Vec<Point3<f32>>,
    pub index: Vec<u32>,
    pub transform: Matrix4<f32>,
}

pub struct Model {
    vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
    normals: Arc<CpuAccessibleBuffer<[Normal]>>,
    index: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Model {
    pub fn new(vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
                normals: Arc<CpuAccessibleBuffer<[Normal]>>,
                index: Arc<CpuAccessibleBuffer<[u32]>>) -> Model {
        Model {
            vertex: vertex,
            normals: normals,
            index: index,
        }
    }

    pub fn draw_indexed<S>(&self,
        builder: &mut AutoCommandBufferBuilder<StandardCommandPoolBuilder>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        set: S
        )
        where S: DescriptorSetsCollection, {
        builder.draw_indexed(
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
        return MyMesh::from_gltf(path).get_buffers(device);
    }
}

impl MyMesh {
    fn new(vertex: Vec<cgmath::Point3<f32>>,
           normals: Vec<cgmath::Point3<f32>>,
           index: Vec<u32>,
           transform: Matrix4<f32>) -> MyMesh {
        MyMesh {
            vertex: vertex,
            normal: normals,
            index: index,
            transform: transform,
        }
    }

    pub fn from_gltf(path: &Path) -> MyMesh {
        let (d, b, _i) = gltf::import(path).unwrap();
        let mesh = d.meshes().next().unwrap();
        let primitive = mesh.primitives().next().unwrap();
        let reader = primitive.reader(|buffer| Some(&b[buffer.index()]));
        let vertex = {
            let iter = reader
                .read_positions()
                .expect(&format!(
                    "primitives must have the POSITION attribute (mesh: {}, primitive: {})",
                    mesh.index(), primitive.index()));

            iter
            .map(|arr| {
                println!("p {:?}", arr);
                Point3::from(arr)
            })
            .collect::<Vec<_>>()
        };
        let normals = {
            let iter = reader
                .read_normals()
                .expect(&format!(
                    "primitives must have the NORMALS attribute (mesh: {}, primitive: {})",
                    mesh.index(), primitive.index()));
            iter
            .map(|arr| {
                println!("n {:?}", arr);
                Point3::from(arr)
            })
            .collect::<Vec<_>>()
        };
        let index = reader
            .read_indices()
            .map(|read_indices| {
                read_indices.into_u32().collect::<Vec<_>>()
            });

        let node: Node = d.nodes().filter(|node| node.mesh().is_some()).next().unwrap();
        let transform = Matrix4::from(node.transform().matrix());
        let (translation, rotation, scale) = node.transform().decomposed();
        println!("t {:?} r {:?} s {:?}", translation, rotation, scale);

        return MyMesh::new(vertex, normals, index.unwrap(), transform);
    }

    pub fn get_buffers(&self, device: &Arc<Device>) -> Model {
        let vertices_vec: Vec<Vertex> = self.vertex.iter()
        .map(|pos| self.transform.transform_point(*pos))
        .map(|pos| Vertex{position: (pos[0], pos[1], pos[2])}).collect();
        let vertices = vertices_vec.iter().cloned();
        let normals_vec: Vec<Normal> = 
        self.normal.iter()
        .map(|pos| self.transform.transform_point(*pos))
        .map(|pos| Normal{normal: (pos[0], pos[1], pos[2])})
        .collect();
        let normals = normals_vec.iter().cloned();

        let indices = self.index.iter().cloned();

        println!("mesh properties: vertices {} normals {} indices {}", vertices_vec.len(), normals_vec.len(), self.index.len());


        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vertices)
            .unwrap();
        let index_buffer =
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, indices)
            .unwrap();

        let normals_buffer =
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, normals)
            .unwrap();
        Model::new(vertex_buffer, normals_buffer, index_buffer)
    }
}
