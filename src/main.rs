mod ball;
mod sprite;
mod paddle;

use wgpu as wg;
use wg::util::DeviceExt;
use glfw::Context;

use glam::Vec2;
use crate::sprite::Sprite;
use crate::paddle::Paddle;

const VERTICES: &[f32] = &[
    -0.5, -0.5,
    0.5, -0.5,
    0.5, 0.5,
    -0.5, 0.5,
];
const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];


#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut glfw = glfw::init_no_callbacks().unwrap();
    glfw.set_swap_interval(glfw::SwapInterval::None);
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::ScaleToMonitor(true));
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

    let (mut window, events) = glfw.create_window(600, 600, "Pong", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    window.set_key_polling(true);

    let (width, height) = window.get_framebuffer_size();
    let config = wg::SurfaceConfiguration {
        usage: wg::TextureUsages::RENDER_ATTACHMENT,
        alpha_mode: wg::CompositeAlphaMode::Opaque,
        format: wg::TextureFormat::Bgra8UnormSrgb,
        present_mode: wg::PresentMode::Fifo,
        desired_maximum_frame_latency: 1,
        height: height as u32,
        width: width as u32,
        view_formats: vec![],
    };

    let instance = wg::Instance::new(wg::InstanceDescriptor{
        backends: wg::Backends::PRIMARY,
        dx12_shader_compiler: wg::Dx12Compiler::Fxc,
        flags: wg::InstanceFlags::from_build_config(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic
    });

    let surface = instance.create_surface(window.render_context()).unwrap();
    let adapter = instance.request_adapter(&wg::RequestAdapterOptions {
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
        power_preference: wg::PowerPreference::HighPerformance,
    }).await.unwrap();
    let (device, queue) = adapter.request_device(&wg::DeviceDescriptor {
        label: Some("Pong device"),
        required_features: wg::Features::empty(),
        required_limits: wg::Limits::default(),
    }, None).await.unwrap();
    surface.configure(&device, &config);

    let sprite_layout = device.create_bind_group_layout(&wg::BindGroupLayoutDescriptor {
        label: Some("Sprite Bind Group Layout"),
        entries: &[
            wg::BindGroupLayoutEntry {
                binding: 0,
                visibility: wg::ShaderStages::VERTEX,
                ty: wg::BindingType::Buffer {
                    ty: wg::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let module = device.create_shader_module(wg::include_wgsl!("shaders/game.wgsl"));

    let pipeline = device.create_render_pipeline(&wg::RenderPipelineDescriptor {
        label: Some("Pong game pipeline"),
        layout: Some(&device.create_pipeline_layout(&wg::PipelineLayoutDescriptor {
            label: Some("Pong game pipeline layout"),
            bind_group_layouts: &[&sprite_layout],
            push_constant_ranges: &[],
        })),

        multiview: None,
        depth_stencil: None,
        multisample: Default::default(),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        vertex: wg::VertexState {
            module: &module,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                step_mode: wgpu::VertexStepMode::Vertex,
                array_stride: 2 * std::mem::size_of::<f32>() as u64,
                attributes: &[wg::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wg::VertexFormat::Float32x2,
                }],
            }],
        },
        fragment: Some(wg::FragmentState {
            module: &module,
            entry_point: "fs_main",
            targets: &[Some(wg::ColorTargetState {
                format: config.format,
                blend: Some(wg::BlendState {
                    color: wg::BlendComponent {
                        operation: wg::BlendOperation::Add,
                        src_factor: wg::BlendFactor::SrcAlpha,
                        dst_factor: wg::BlendFactor::OneMinusSrcAlpha,
                    },
                    alpha: wg::BlendComponent {
                        src_factor: wg::BlendFactor::One,
                        dst_factor: wg::BlendFactor::One,
                        operation: wg::BlendOperation::Add,
                    },
                }),
                write_mask: wg::ColorWrites::ALL,
            })],
        }),
    });

    let quad_vbo = device.create_buffer_init(&wg::util::BufferInitDescriptor {
        label: Some("Quad vertex buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wg::BufferUsages::VERTEX,
    });
    let quad_ibo = device.create_buffer_init(&wg::util::BufferInitDescriptor {
        label: Some("Quad index buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wg::BufferUsages::INDEX,
    });

    let ball_sprite = Sprite::new(&device, &queue, Vec2::ZERO, Vec2::new(0.05, 0.05), &sprite_layout);
    let mut ball = ball::Ball::new(ball_sprite, 2.0);
    let paddle1_sprite = Sprite::new(&device, &queue, Vec2::new(-1.0, 0.0), Vec2::new(0.1, 0.35), &sprite_layout);
    let mut paddle1 = Paddle::new(paddle1_sprite, 3.0);
    let paddle2_sprite = Sprite::new(&device, &queue, Vec2::new(1.0, 0.0), Vec2::new(0.1, 0.35), &sprite_layout);
    let mut paddle2 = Paddle::new(paddle2_sprite, 3.0);

    let mut player1_score = 0;
    let mut player2_score = 0;
    let mut server = 1.0;
    let mut time = glfw.get_time();
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Press, _) => {
                    server *= -1.0;
                    ball.velocity = Vec2::new(1.0, 0.35) * server;
                    ball.sprite.position = Vec2::ZERO;
                }
                glfw::WindowEvent::Key(glfw::Key::W, _, action, _) => {
                    if action == glfw::Action::Press {
                        paddle1.wish_dir.y += 1.0;
                    } else if action == glfw::Action::Release {
                        paddle1.wish_dir.y -= 1.0;
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::S, _, action, _) => {
                    if action == glfw::Action::Press {
                        paddle1.wish_dir.y += -1.0;
                    } else if action == glfw::Action::Release {
                        paddle1.wish_dir.y -= -1.0;
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::Up, _, action, _) => {
                    if action == glfw::Action::Press {
                        paddle2.wish_dir.y += 1.0;
                    } else if action == glfw::Action::Release {
                        paddle2.wish_dir.y -= 1.0;
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::Down, _, action, _) => {
                    if action == glfw::Action::Press {
                        paddle2.wish_dir.y += -1.0;
                    } else if action == glfw::Action::Release {
                        paddle2.wish_dir.y -= -1.0;
                    }
                }
                _ => {}
            }
        }
        let dt = glfw.get_time() - time;
        time = glfw.get_time();
        paddle1.update(dt as f32, &queue);
        paddle2.update(dt as f32, &queue);
        ball.update(dt as f32, &queue, &[&paddle1, &paddle2]);

        // check if a player scored
        if ball.sprite.position.x <= -0.95 {
            player2_score += 1;
            ball.velocity = Vec2::ZERO;
            ball.sprite.position = Vec2::ZERO;
            println!("Player 1: {} - Player 2: {}", player1_score, player2_score);
        } else if ball.sprite.position.x >= 0.95 {
            player1_score += 1;
            ball.velocity = Vec2::ZERO;
            ball.sprite.position = Vec2::ZERO;
            println!("Player 1: {} - Player 2: {}", player1_score, player2_score);
        }

        let swapchain = surface.get_current_texture().unwrap();
        let frame = swapchain.texture.create_view(&wg::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wg::CommandEncoderDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wg::RenderPassDescriptor {
                label: Some("Pong game pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
                color_attachments: &[Some(wg::RenderPassColorAttachment {
                    view: &frame,
                    resolve_target: None,
                    ops: wg::Operations {
                        store: wg::StoreOp::Store,
                        load: wg::LoadOp::Clear(wg::Color::BLACK),
                    },
                })],
            });
            rpass.set_pipeline(&pipeline);
            rpass.set_vertex_buffer(0, quad_vbo.slice(..));
            rpass.set_index_buffer(quad_ibo.slice(..), wg::IndexFormat::Uint16);

            rpass.set_bind_group(0, &paddle1.sprite.group, &[]);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

            rpass.set_bind_group(0, &paddle2.sprite.group, &[]);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

            rpass.set_bind_group(0, &ball.sprite.group, &[]);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
        queue.submit([encoder.finish()]);
        swapchain.present();
        window.swap_buffers();
    }
}
