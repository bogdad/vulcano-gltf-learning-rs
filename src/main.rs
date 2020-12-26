use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano_win::VkSurfaceBuild;

use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain,
    SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano::image::{ImageUsage, SwapchainImage, AttachmentImage};
use vulkano::format::Format;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode};

use cgmath::prelude::*;
use cgmath::{Matrix4, Matrix3, Vector3, Point3, Rad};


use std::iter;
use std::sync::Arc;
use std::path::Path;
use std::time::Instant;

mod render;
mod utils;

use render::Model;
use utils::{Vertex, Normal};


mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    //v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_normal = mat3(worldview) * normal;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}

        "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(0.0, 0.0, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_color = vec3(0.6, 0.6, 0.6);
    vec3 regular_color = vec3(1.0, 1.0, 1.0);

    f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
       "
    }
}

fn main() {
    println!("Hello, world!");
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &required_extensions, None).unwrap();
    for device in PhysicalDevice::enumerate(&instance) {
        println!(
            "possible device: {} (type: {:?})",
            device.name(),
            device.ty()
        );      
    };
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();
    let dimensions: [u32; 2] = surface.window().inner_size().into();
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .unwrap();
    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    ).unwrap();
    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .unwrap()
    };

    let models = vec![
        //Model::from_gltf(Path::new("creature.glb"), &device),
        //Model::from_gltf(Path::new("creature2.glb"), &device),
        //Model::from_gltf(Path::new("creature3.glb"), &device),
        //Model::from_gltf(Path::new("dog.glb"), &device),
        Model::from_gltf(Path::new("box.glb"), &device),
        Model::from_gltf(Path::new("center.glb"), &device),
    ];
    

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    let vs = vs::Shader::load(device.clone()).unwrap();
    //let tcs = tcs::Shader::load(device.clone()).unwrap();
    //let tes = tes::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    // gltf: 
    // "and the default camera sits on the 
    // -Z side looking toward the origin with +Y up"
    //                               x     y    z
    let mut camera_pos = Point3::new(0.0, 0.0, -1.0);                          
    let camera_front = Vector3::new(0.0, 0.0, 1.0);
    let camera_up = Vector3::new(0.0, 1.0, 0.0);


    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap(),
    );

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
    /*let dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
        compare_mask: None,
        write_mask: None,
        reference: None,
    };*/

    let (mut pipeline, mut framebuffers) =
        window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());

    let rotation_start = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            recreate_swapchain = true;
        }
        Event::WindowEvent {
             event: WindowEvent::KeyboardInput {
                 input: KeyboardInput { virtual_keycode: Some(key_code), .. }, ..
             },
             ..
        } => {
            let camera_speed = 0.25;
            let zz = camera_front.cross(camera_up).normalize();
            match key_code {
                 VirtualKeyCode::A => camera_pos -= zz * camera_speed,
                 VirtualKeyCode::D => camera_pos += zz * camera_speed,
                 VirtualKeyCode::W => camera_pos += camera_speed * camera_front,
                 VirtualKeyCode::S => camera_pos -= camera_speed * camera_front,
                 _ => {}
            }
        }
        Event::RedrawEventsCleared => {
            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if recreate_swapchain {
                let dimensions: [u32; 2] = surface.window().inner_size().into();
                let (new_swapchain, new_images) =
                    match swapchain.recreate_with_dimensions(dimensions) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::UnsupportedDimensions) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                    };

                swapchain = new_swapchain;
                let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                        device.clone(),
                        &vs,
                        &fs,
                        &new_images,
                        render_pass.clone(),
                    );
                pipeline = new_pipeline;
                framebuffers = new_framebuffers;
                recreate_swapchain = false;
            }
            let uniform_buffer_subbuffer = {
                let elapsed = rotation_start.elapsed();
                let rotation = 
                    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
                let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

                // note: this teapot was meant for OpenGL where the origin is at the lower left
                //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
                let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                let proj = cgmath::perspective(
                    Rad(std::f32::consts::FRAC_PI_2),
                    aspect_ratio,
                    0.01,
                    100.0,
                );
                
                let target = camera_pos.to_vec() + camera_front;

                let view = Matrix4::look_at(
                    camera_pos, Point3::from_vec(target), camera_up
                );
                let scale = Matrix4::from_scale(0.01);
                /*
                    mat4 worldview = uniforms.view * uniforms.world;
                    v_normal = transpose(inverse(mat3(worldview))) * normal;
                    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
                 */
                let uniform_data = vs::ty::Data {
                    //world: Matrix4::from(eye).into(),
                    world: Matrix4::from(rotation).into(),
                    //world: <Matrix4<f32> as One>::one().into(),
                    view: (view * scale).into(),
                    proj: proj.into(),
                };

                uniform_buffer.next(uniform_data).unwrap()
            };
            let layout = pipeline.descriptor_set_layout(0).unwrap();
            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            let (image_num, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            let mut builder =
                AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(), queue.family())
                    .unwrap();
            builder
                .begin_render_pass(
                    framebuffers[image_num].clone(),
                    SubpassContents::Inline,
                    vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
                )
                .unwrap();
            for model in &models {
                model.draw_indexed(&mut builder, pipeline.clone(), set.clone())
            }
            
            builder.end_render_pass()
                .unwrap();
            let command_buffer = builder.build().unwrap();

            let future = previous_frame_end
                .take()
                .unwrap()
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffer)
                .unwrap()
                .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Some(future.boxed());
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    previous_frame_end = Some(sync::now(device.clone()).boxed());
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    previous_frame_end = Some(sync::now(device.clone()).boxed());
                }
            }
        }
        _ => (),
    });
}




fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> (
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
) {
    let dimensions = images[0].dimensions();

    let depth_buffer =
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>();

    // In the triangle example we use a dynamic viewport, as its a simple example.
    // However in the teapot example, we recreate the pipelines with a hardcoded viewport instead.
    // This allows the driver to optimize things, at the cost of slower window resizes.
    // https://computergraphics.stackexchange.com/questions/5742/vulkan-best-way-of-updating-pipeline-viewport
    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input(TwoBuffersDefinition::<Vertex, Normal>::new())
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }))
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    (pipeline, framebuffers)
}

