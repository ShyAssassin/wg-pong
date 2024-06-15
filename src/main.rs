use wgpu as wg;
use glfw::Context;


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

    let module = device.create_shader_module(wg::include_wgsl!("shaders/game.wgsl"));

    let pipeline = device.create_render_pipeline(&wg::RenderPipelineDescriptor {
        label: Some("Pong game pipeline"),
        layout: None,
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
            buffers: &[],
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

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
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
            rpass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        swapchain.present();
        window.swap_buffers();
    }
}
