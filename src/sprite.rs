use glam::Vec2;

#[derive(Debug)]
pub struct Sprite {
    pub size: Vec2,
    pub position: Vec2,
    pub group: wgpu::BindGroup,
    pub transform_buffer: wgpu::Buffer,
}

impl Sprite {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, position: Vec2, scale: Vec2, layout: &wgpu::BindGroupLayout) -> Self {
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transform Buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&transform_buffer, 0, bytemuck::cast_slice(&[
            glam::Mat4::from_translation((position, 0.0).into()) * glam::Mat4::from_scale((scale, 1.0).into())
        ]));

        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &transform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: Some("Sprite Bind Group"),
        });

        Self {
            size: scale,
            position,
            group,
            transform_buffer,
        }
    }

    pub fn is_coliding(&self, other: &Sprite) -> bool {
        let self_min = self.position - self.size / 2.0;
        let self_max = self.position + self.size / 2.0;
        let other_min = other.position - other.size / 2.0;
        let other_max = other.position + other.size / 2.0;

        self_min.x < other_max.x && self_max.x > other_min.x && self_min.y < other_max.y && self_max.y > other_min.y
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.transform_buffer, 0, bytemuck::cast_slice(&[
            glam::Mat4::from_translation((self.position, 0.0).into()) * glam::Mat4::from_scale((self.size, 1.0).into())
        ]));
    }
}
