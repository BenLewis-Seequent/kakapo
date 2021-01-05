use wgpu::util::{DeviceExt, StagingBelt};
use wgpu::RenderPass;

use crate::geom::Rect;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Quad {
    position: [f32; 2],
    size: [f32; 2],
    colour: [f32; 4],
}

const QUAD_SIZE: wgpu::BufferSize =
    unsafe { wgpu::BufferSize::new_unchecked(std::mem::size_of::<Quad>() as u64) };

const INDICES: &[u16] = &[0, 1, 2, 3];

const VERTICES: &[[f32; 2]] = &[[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

pub(super) struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    number_of_quads: u32,
}

macro_rules! shader {
    ($device:expr, $spv:tt) => {
        $device.create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/", $spv)))
    };
}

impl QuadPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> QuadPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let vs_module = shader!(device, "quad.vert.spv");
        let fs_module = shader!(device, "quad.frag.spv");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float2],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: QUAD_SIZE.get(),
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float2,
                                                          2 => Float2,
                                                          3 => Float4],
                    },
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Instance"),
            size: QUAD_SIZE.get() * 128,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        QuadPipeline {
            pipeline: render_pipeline,
            index_buffer,
            vertex_buffer,
            instance_buffer,
            number_of_quads: 0,
        }
    }

    pub fn reset(&mut self) {
        self.number_of_quads = 0
    }

    pub fn add_quad(
        &mut self,
        belt: &mut StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        rect: Rect,
        colour: [f32; 4],
    ) {
        let mut buffer = belt.write_buffer(
            encoder,
            &self.instance_buffer,
            self.number_of_quads as u64 * QUAD_SIZE.get(),
            QUAD_SIZE,
            device,
        );
        buffer.copy_from_slice(bytemuck::bytes_of(&Quad {
            position: [rect.origin.x, rect.origin.y],
            size: [rect.size.width, rect.size.height],
            colour,
        }));
        self.number_of_quads += 1;
    }

    pub fn record<'a>(&'a self, encoder: &mut RenderPass<'a>) {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_index_buffer(self.index_buffer.slice(..));
        encoder.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        encoder.set_vertex_buffer(1, self.instance_buffer.slice(..));
        encoder.draw_indexed(0..4, 0, 0..self.number_of_quads)
    }
}
