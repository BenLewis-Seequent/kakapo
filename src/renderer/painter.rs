use crate::renderer::Renderer;
use crate::geom::{Rect, Position, Size};

pub struct Painter<'a> {
    renderer: &'a mut Renderer,
    encoder: &'a mut wgpu::CommandEncoder,
    viewport_size: Size,
    origin: Position,
}

impl<'a> Painter<'a> {
    pub(super) fn new(renderer: &'a mut Renderer, encoder: &'a mut wgpu::CommandEncoder, viewport_size: Size) -> Self {
        renderer.quad.reset();
        Painter {
            renderer,
            encoder,
            viewport_size,
            origin: Position::zero()
        }
    }

    /// Transforms the rect from widget space into Vulkan coordinate space, where the viewport goes
    /// from -1 to 1.
    fn transform_rect(&self, mut rect: Rect) -> Rect {
        rect.origin += self.origin;
        Rect::new(
            Position::new(
                2.0 * rect.origin.x / self.viewport_size.width - 1.0,
                1.0 - 2.0 * (rect.origin.y + rect.size.height) / self.viewport_size.height,
            ),
            Size::new(2.0 * rect.size.width / self.viewport_size.width,
                          2.0 * rect.size.height / self.viewport_size.height)
        )
    }

    pub fn paint_quad(&mut self, rect: Rect, colour: [f32; 4]) {
        let transformed_rect = self.transform_rect(rect);
        self.renderer.quad.add_quad(&mut self.renderer.belt,
                                    &mut self.encoder,
                                    &self.renderer.device,
                                    transformed_rect, colour);
    }

    pub fn with_rect(&mut self, rect: Rect) -> Painter<'_> {
        Painter {
            renderer: self.renderer,
            encoder: self.encoder,
            viewport_size: self.viewport_size,
            origin: self.origin + rect.origin,
        }
    }
}
