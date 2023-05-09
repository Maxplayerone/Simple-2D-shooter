use cgmath::{ortho, Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    //view_matrix: Matrix4<f32>,
    //projection_matrix: Matrix4<f32>,
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let left = 0.0;
        let right = config.width as f32;
        let bottom = 0.0;
        let top = config.height as f32;
        let near = 0.1;
        let far = 100.0;
        let projection_matrix = ortho(left, right, bottom, top, near, far);

        let eye = Point3::new(0.0, 0.0, 1.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let view_matrix = Matrix4::look_at_rh(eye, target, up);

        let view_proj_mat = OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;

        Self {
            //view_matrix,
            //projection_matrix,
            view_proj: view_proj_mat.into(),
        }
    }
}

pub struct Camera {
    //camera_uniform: CameraUniform,
    //camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let camera_uniform = CameraUniform::new(config);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera bind group layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera bind group"),
        });

        Self {
            //camera_uniform,
            //camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }
}
