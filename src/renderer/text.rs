use wgpu::util::StagingBelt;
use wgpu::{CommandEncoder, SwapChainTexture};
use wgpu_glyph::GlyphBrush;

pub(super) struct TextPipeline {
    glyph_brush: GlyphBrush<()>,
}

impl TextPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> TextPipeline {
        let font =
            wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("Roboto-Regular.ttf"))
                .unwrap();

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(font).build(device, format);

        TextPipeline { glyph_brush }
    }

    pub fn add_text(&mut self, section: wgpu_glyph::Section) {
        self.glyph_brush.queue(section);
    }

    pub fn record(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut StagingBelt,
        encoder: &mut CommandEncoder,
        frame: &SwapChainTexture,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        self.glyph_brush
            .draw_queued(
                device,
                staging_belt,
                encoder,
                &frame.view,
                size.width,
                size.height,
            )
            .expect("Draw queued");
    }
}
