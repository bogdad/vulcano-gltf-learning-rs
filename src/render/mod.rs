pub mod gltfimporter;
pub mod model;
pub mod mymesh;
pub mod scene;
pub mod skybox;
pub mod system;
pub mod textures;

use cgmath::{Point2, Matrix4, Point3};

pub type Vertex = Vec<Point3<f32>>;
pub type Normals = Vec<Point3<f32>>;
pub type Tex = Vec<Point2<f32>>;
pub type TexOffset = Vec<Point2<i32>>;
pub type Index = Vec<u32>;
pub type Trans = Matrix4<f32>;
pub type InvTrans = Matrix4<f32>;