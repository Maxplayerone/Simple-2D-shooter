use wgpu::util::DeviceExt;

pub struct Renderer{
    pub render_pipeline: wgpu::RenderPipeline,
}

pub struct Buffers{
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_of_indices: u32,
    vertices: [Vertex; 4],
    indices: [u16; 6],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>{
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute{
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute{
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}


impl Renderer{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, shader: &wgpu::ShaderModule, camera_bind_group: wgpu::BindGroupLayout) -> Self{
        //describes available binding group of the pipeline 
        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&camera_bind_group],
                push_constant_ranges: &[],
            });
        //describes shaders, buffers and stuff
        let render_pipeline = 
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
                label: Some("Render pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState{
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()], //this is not the contents of vertex buffers but how vertex data is laid out (VertexBufferLayout)
                },
                fragment: Some(wgpu::FragmentState{
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState{
                        format: config.format,
                        blend: Some(wgpu::BlendState{
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState{
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState{
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        
        Self{
            render_pipeline,
        }
    }
    
    pub fn push_quad(device: &wgpu::Device, x: f32, y: f32) -> Buffers{
       
        let vertices = [
            Vertex{position: [x, y], color: [0.0, 1.0, 0.0]},
            Vertex{position: [x + 100.0, y], color: [0.0, 1.0, 0.0]},
            Vertex{position: [x, y + 100.0], color: [0.0, 1.0, 0.0]},
            Vertex{position: [x + 100.0, y + 100.0], color: [0.0, 1.0, 0.0]},
        ];
        
        let indices = [0, 1, 2, 2, 1, 3];
               
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        let num_of_indices = 6;
        
        Buffers{
            vertex_buffer,
            index_buffer,
            num_of_indices,
            vertices,
            indices,
        }
    }
    
}