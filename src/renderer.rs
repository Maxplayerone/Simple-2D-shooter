use cgmath::{point2, point3, Point2, Point3};
use wgpu::util::DeviceExt;

pub struct Renderer {
    pub render_pipeline: wgpu::RenderPipeline,

    prev_quad_positions: Vec<Point2<f32>>,
    quad_size: f32,
    current_quad_index: usize,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

pub struct Buffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_of_indices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn new(position: Point2<f32>, color: Point3<f32>) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
        }
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
        camera_bind_group: &wgpu::BindGroupLayout,
        size: f32,
    ) -> Self {
        //describes available binding group of the pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[camera_bind_group],
                push_constant_ranges: &[],
            });
        //describes shaders, buffers and stuff
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()], //this is not the contents of vertex buffers but how vertex data is laid out (VertexBufferLayout)
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
            prev_quad_positions: Vec::new(),
            quad_size: size,
            vertices: Vec::new(),
            indices: Vec::new(),
            current_quad_index: 0,
        }
    }

    pub fn create_quad_data(&mut self, position: Point2<f32>) -> usize {

        self.prev_quad_positions.push(position);
        let index = self.current_quad_index;
        self.current_quad_index = self.current_quad_index + 1;

        self.vertices
            .push(Vertex::new(position, point3::<f32>(0.0, 1.0, 0.0)));
        self.vertices.push(Vertex::new(
            point2::<f32>(position.x + self.quad_size, position.y),
            point3::<f32>(0.0, 1.0, 0.0),
        ));
        self.vertices.push(Vertex::new(
            point2::<f32>(position.x, position.y + self.quad_size),
            point3::<f32>(0.0, 1.0, 0.0),
        ));
        self.vertices.push(Vertex::new(
            point2::<f32>(position.x + self.quad_size, position.y + self.quad_size),
            point3::<f32>(0.0, 1.0, 0.0),
        ));

        self.indices.push((index * 4).try_into().unwrap());
        self.indices.push((index * 4 + 1).try_into().unwrap());
        self.indices.push((index * 4 + 2).try_into().unwrap());
        self.indices.push((index * 4 + 2).try_into().unwrap());
        self.indices.push((index * 4 + 1).try_into().unwrap());
        self.indices.push((index * 4 + 3).try_into().unwrap());

        index
    }

    pub fn update_quad_data(&mut self, index: usize, delta_position: Point2<f32>) {
        let prev_pos = self.prev_quad_positions[index];
        let new_quad_pos =
            point2::<f32>(prev_pos.x + delta_position.x, prev_pos.y + delta_position.y);
        let dummy_color = point3::<f32>(0.0, 1.0, 0.0);

        self.vertices[4 * index] = Vertex::new(new_quad_pos, dummy_color);
        self.vertices[4 * index + 1] = Vertex::new(
            point2::<f32>(new_quad_pos.x + self.quad_size, new_quad_pos.y),
            dummy_color,
        );
        self.vertices[4 * index + 2] = Vertex::new(
            point2::<f32>(new_quad_pos.x, new_quad_pos.y + self.quad_size),
            dummy_color,
        );
        self.vertices[4 * index + 3] = Vertex::new(
            point2::<f32>(
                new_quad_pos.x + self.quad_size,
                new_quad_pos.y + self.quad_size,
            ),
            dummy_color,
        );

        self.prev_quad_positions[index] = new_quad_pos;
    }

    pub fn collect_buffers(&mut self, device: &wgpu::Device) -> Buffers {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_of_indices = (6 * self.current_quad_index).try_into().unwrap();

        Buffers {
            vertex_buffer,
            index_buffer,
            num_of_indices 
        }
    }
}
