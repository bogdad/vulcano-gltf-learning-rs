pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec2 tex_offset;
layout(location = 3) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_tex;
layout(location = 2) out vec2 v_tex_offset;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;



void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_tex = tex;
    v_tex_offset = tex_offset;
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

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_tex2;
layout(location = 2) in vec2 v_tex_offset2;

layout(location = 0) out vec4 f_color;

const vec3 LIGHT_VEC = vec3(0.0, 0.0, 1.0);

layout(set = 0, binding = 1) uniform sampler2D textureSrc;
layout(std140, set = 0, binding = 2) uniform PointLights {
    PointLight plight[128];
};

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT_VEC));
    vec3 dark_color = vec3(0.6, 0.6, 0.6);
    vec3 regular_color = vec3(1.0, 1.0, 1.0);
    if (v_tex2.x < 0 || v_tex2.y < 0) {
      f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
    } else {
      f_color = texture(textureSrc, v_tex2);
      if (f_color.r < 0.1) {
        discard;
      }
    }
}
       "
  }
}
