use crate::shaders::main::fs::ty::{DirectionalLight, PointLight, SpotLight};

use std::sync::Arc;

#[derive(Default, Clone)]
pub struct Scene {
  pub point_lights: Vec<Arc<PointLight>>,
  pub directional_lights: Vec<Arc<DirectionalLight>>,
  pub spot_lights: Vec<Arc<SpotLight>>,
}

#[derive(Default, Clone)]
pub struct MergedScene {
  pub point_lights: Vec<PointLight>,
  pub directional_lights: Vec<DirectionalLight>,
  pub spot_lights: Vec<SpotLight>,
}
