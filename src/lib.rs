use std::iter;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod renderer;
use renderer::Renderer;

mod camera;
use camera::Camera;

mod physics;
use physics::check_player_gravity_collission;

use cgmath::{point2, point3, Point2};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    renderer: Renderer,
    camera: Camera,
    player: Player,
    block: Block,

    quad_size: usize,
}

pub trait Entity {
    fn get_id(&self) -> usize;
    fn get_pos(&self) -> &Point2<f32>;
}

struct Player {
    index: usize,
    pos: Point2<f32>,

    speed: f32,
    is_left_pressed: bool,
    is_right_pressed: bool,
    gravity: f32,
}

impl Entity for Player {
    fn get_id(&self) -> usize {
        self.index
    }

    fn get_pos(&self) -> &Point2<f32> {
        &self.pos
    }
}

struct Block {
    index: usize,
    pos: Point2<f32>,
}

impl Entity for Block {
    fn get_id(&self) -> usize {
        self.index
    }

    fn get_pos(&self) -> &Point2<f32> {
        &self.pos
    }
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera = Camera::new(&config, &device);

        let quad_size = 50;
        let player_pos = point2::<f32>(config.width as f32 / 2.0, config.height as f32 / 2.0);
        let block_pos = point2::<f32>(200.0, 230.0);

        let mut renderer = Renderer::new(
            &device,
            &config,
            &shader,
            camera.bind_group_layout(),
            quad_size,
        );
        let index = renderer.create_quad(player_pos, point3::<f32>(0.0, 1.0, 0.0));
        let block_index = renderer.create_block(
            block_pos,
            point2::<usize>(6, 1),
            point3::<f32>(1.0, 1.0, 1.0),
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            renderer,
            camera,
            player: Player {
                speed: 5.0,
                is_left_pressed: false,
                is_right_pressed: false,
                gravity: 3.0,
                index,
                pos: player_pos,
            },
            block: Block {
                index: block_index,
                pos: block_pos,
            },
            quad_size,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::A => {
                        self.player.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.player.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        //horizontal movement
        if self.player.is_right_pressed {
            self.renderer
                .update_quad_data(self.player.index, point2::<f32>(self.player.speed, 0.0));
            self.player.pos =
                point2::<f32>(self.player.pos.x + self.player.speed, self.player.pos.y);
        }
        if self.player.is_left_pressed {
            self.renderer
                .update_quad_data(self.player.index, point2::<f32>(-self.player.speed, 0.0));
            self.player.pos =
                point2::<f32>(self.player.pos.x - self.player.speed, self.player.pos.y);
        }

        //vertical movement
        //GRAVITY
        let player_pos_after_gravity =
            point2::<f32>(self.player.pos.x, self.player.pos.y - self.player.gravity);
        match check_player_gravity_collission(
            player_pos_after_gravity,
            self.block.pos,
            self.quad_size,
        ) {
            Some(new_player_pos) => {
                self.renderer.change_quad_data(self.player.index, new_player_pos);
                self.player.pos = new_player_pos;
                },
            None => {
                self.renderer
                    .update_quad_data(self.player.index, point2::<f32>(0.0, -self.player.gravity));
                self.player.pos = player_pos_after_gravity;
            }
        };
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        //saves a series of gpu instructions (for example render_pass)
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let buffers = self.renderer.collect_buffers(&self.device);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            b: 0.2,
                            g: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.renderer.render_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..buffers.num_of_indices, 0, 0..1);
        }
        self.queue.submit(iter::once(encoder.finish()));
        output.present(); //draws the stuff to the surface texture
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
