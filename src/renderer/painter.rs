use crate::renderer::Renderer;
use crate::geom::Rect;

pub struct Painter<'a> {
    renderer: &'a mut Renderer,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> Painter<'a> {
    pub(super) fn new(renderer: &'a mut Renderer, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        renderer.quad.reset();
        Painter {
            renderer,
            encoder
        }
    }

    pub fn paint_quad(&mut self, rect: Rect, colour: [f32; 4]) {
        self.renderer.quad.add_quad(&mut self.renderer.belt,
                                    &mut self.encoder,
                                    &self.renderer.device,
                                    rect, colour);
    }
}
