// https://github.com/adrien-ben/gltf-viewer-rs/blob/master/assets/shaders/skybox.vert
// https://github.com/adrien-ben/gltf-viewer-rs/blob/master/assets/shaders/skybox.frag

pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      src: "
#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec2 tex_offset;
layout(location = 3) in vec3 normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;


void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
"
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      src: "

#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput inputColor;
layout (input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput inputDepth;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = subpassLoad(inputColor).rgb;

    outColor = vec4(color, 1.0);
}
       ",
       types_meta: {
        #[derive(Clone, Copy, PartialEq, Debug, Default)]

        impl Eq
    }
  }
}
