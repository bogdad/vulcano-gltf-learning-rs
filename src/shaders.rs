

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
layout(location = 3) out vec3 v_position;

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
    v_position = position;
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

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_tex2;
layout(location = 2) in vec2 v_tex_offset2;
layout(location = 3) in vec3 v_position2;

layout(location = 0) out vec4 f_color;

const vec3 LIGHT_VEC = vec3(0.0, 0.0, 1.0);

layout(set = 0, binding = 1) uniform sampler2D textureSrc;

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 color;
    float intensity;
    vec3 direction;
};

struct SpotLight {
    vec3 position;
    vec3 color;
    vec3 direction;
    float angle;
    float intensity;
    float range;
    float smoothness;
};

layout(std140, set = 0, binding = 2) uniform Environment {
    vec3 ambient_color;
    vec3 camera_position;
    int point_light_count;
    int directional_light_count;
    int spot_light_count;
    // https://github.com/jwagner/webglice/blob/master/shaders/sun.glsl
    vec3 sun_color;
    vec3 sun_direction;
};

layout(std140, set = 0, binding = 3) uniform PointLights {
    PointLight plight[128];
};

layout(std140, set = 0, binding = 4) uniform DirectionalLights {
    DirectionalLight dlight[16];
};

layout(std140, set = 0, binding = 5) uniform SpotLights {
    SpotLight slight[128];
};

// https://github.com/jwagner/webglice/blob/master/shaders/sun.glsl
vec3 sun(const vec3 surface_normal, const vec3 eye_normal, float shiny, float spec, float diffuse){
  vec3 diffuse_color = max(dot(sun_direction, surface_normal), 0.0) * sun_color * diffuse;
  vec3 reflection = normalize(reflect(-sun_direction, surface_normal));
  float direction = max(0.0, dot(eye_normal, reflection));
  vec3 specular = pow(direction, shiny) * sun_color * spec;
  return diffuse_color + specular;
}

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT_VEC));
    vec3 dark_color = vec3(1.0, 1.0, 1.0);
    vec3 regular_color = vec3(1.0, 1.0, 1.0);
    if (v_tex2.x < 0 || v_tex2.y < 0) {

      vec3 lighting = vec3(0.0);
      vec3 normal = normalize(v_normal);
      for (uint i = 0u; i < point_light_count; i++) {
          // Calculate diffuse light
          vec3 light_dir = normalize(plight[i].position - v_position2);
          float diff = max(dot(light_dir, normal), 0.0);
          vec3 diffuse = diff * normalize(plight[i].color);
          // Calculate attenuation
          vec3 dist = plight[i].position - v_position2;
          float dist2 = dot(dist, dist);
          float attenuation = (plight[i].intensity / dist2);
          lighting += diffuse * attenuation;
      }
      for (uint i = 0u; i < directional_light_count; i++) {
          vec3 dir = dlight[i].direction;
          float diff = max(dot(-dir, normal), 0.0);
          vec3 diffuse = diff * dlight[i].color;
          lighting += diffuse * dlight[i].intensity;
      }
      lighting += ambient_color;

      vec3 eye_normal = normalize(camera_position - v_position2);
      vec3 sun_light = sun(normal, eye_normal,  15.0, 2.5, 1.0);
      vec3 color = mix(
        mix(
          mix(dark_color, regular_color, brightness),
          lighting,
          0.5),
        sun_light,
        0.5);
      f_color = vec4(color, 1.0);
    } else {
      f_color = texture(textureSrc, v_tex2);
      if (f_color.r < 0.1) {
        discard;
      }
    }
}
       ",
       types_meta: {
        #[derive(Clone, Copy, PartialEq, Debug, Default)]

        impl Eq
    }
  }
}
