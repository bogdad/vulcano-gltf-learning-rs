#[derive(Default, Copy, Clone, Debug)]
pub struct Vertex {
  pub position: (f32, f32, f32),
}

vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Copy, Clone, Debug)]
pub struct Normal {
  pub normal: (f32, f32, f32),
}

vulkano::impl_vertex!(Normal, normal);

use genmesh::Polygon;
pub type Face = Polygon<u32>;
