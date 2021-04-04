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
    vec3 camera_position;
} uniforms;

layout(location = 0) out vec3 v_position;
layout(location = 1) out vec3 v_position2;
layout(location = 2) out vec3 v_camera_position;

mat4 getViewAtOrigin() {
    mat4 view = mat4(uniforms.view);
    view[3][0] = 0;
    view[3][1] = 0;
    view[3][2] = 0;
    return view;
}

void main() {
    mat4 view = getViewAtOrigin();
    vec4 pos = uniforms.proj * view * vec4(position, 1.0);
    gl_Position = pos;
    v_position = position;
    v_position2 = pos.xyz;
    v_camera_position = uniforms.camera_position;
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

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_position2;
layout(location = 2) in vec3 v_camera_position;

layout (input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput inputColor;
layout (input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput inputDepth;
layout (set = 0, binding = 3) uniform samplerCube cubemapSampler;


layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = subpassLoad(inputColor).rgb;
    if (color.rgb[0] == 0 && color.rgb[1] == 0 && color.rgb[2] == 0) {
      vec3 direction = normalize(-v_position);
      outColor = texture(cubemapSampler, direction);
    } else {
      outColor = vec4(color, 1.0);
    }
}
       ",
       types_meta: {
        #[derive(Clone, Copy, PartialEq, Debug, Default)]

        impl Eq
    }
  }
}
