mod gltfimporter;
mod model;
mod mymesh;
mod scene;
mod skybox;
mod system;
mod textures;

use cgmath::{Point2, Matrix4, Point3};

pub type Vertex = Vec<Point3<f32>>;
pub type Normals = Vec<Point3<f32>>;
pub type Tex = Vec<Point2<f32>>;
pub type TexOffset = Vec<Point2<i32>>;
pub type Index = Vec<u32>;
pub type Trans = Matrix4<f32>;
pub type InvTrans = Matrix4<f32>;

pub use self::gltfimporter::*;
pub use self::model::*;
pub use self::mymesh::*;
pub use self::scene::*;
pub use self::skybox::*;
pub use self::system::*;
pub use self::textures::*;