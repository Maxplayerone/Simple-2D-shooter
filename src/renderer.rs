use std::iter;
use wgpu::util::DeviceExt;

pub struct Renderer{
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
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

const VERTICES: &[Vertex] = &[
    Vertex{
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex{
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex{
        position: [0.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

impl Renderer{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, shader: &wgpu::ShaderModule) -> Self{
        //describes available binding group of the pipeline 
        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[],
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
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        Self{
            render_pipeline,
            vertex_buffer
        }
    }
    
    pub fn render(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), wgpu::SurfaceError> {
         let output = surface.get_current_texture()?;    
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        //saves a series of gpu instructions (for example render_pass)
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Clear(wgpu::Color{
                            r: 0.1,
                            b: 0.2,
                            g: 0.3,
                            a: 1.0
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }
        queue.submit(iter::once(encoder.finish()));
        output.present(); //draws the stuff to the surface texture
        Ok(())
    }    
}