
use winit::window::Window;
use wgpu::util::StagingBelt;
use crate::renderer::painter::Painter;
use crate::renderer::quad::QuadPipeline;
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;

pub mod painter;
mod quad;


pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,

    quad: quad::QuadPipeline,
    belt: StagingBelt,
    pool: LocalPool,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None, // Trace path
        ).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let pipeline = QuadPipeline::new(&device, sc_desc.format);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            quad: pipeline,
            belt: StagingBelt::new(0x100),
            pool: LocalPool::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.recreate();
    }

    pub fn recreate(&mut self) {
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render<F: for<'a> FnOnce(&'a mut Painter<'a>)>(&mut self, render_func: F)
        -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        render_func(&mut Painter::new(self, &mut encoder));

        self.belt.finish();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: None,
            });
            self.quad.record(&mut render_pass);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        self.pool.spawner().spawn_local(self.belt.recall()).unwrap();
        self.pool.run_until_stalled();

        Ok(())
    }
}
