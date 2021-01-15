use std::any::Any;

use crate::events::Event;
use crate::geom::Size;
use crate::renderer::painter::Painter;
use crate::view::{
    UserDataMut, Widget, WidgetCache, WidgetKey, WidgetState, WidgetStateMut, WidgetTree,
};
use crate::Description;
use glyph_brush::Text;

pub trait ButtonDelegate {
    fn pressed(&mut self, parent: UserDataMut<'_>);
}

pub struct Button<D: ButtonDelegate + 'static> {
    colour: [f32; 4],
    delegate: D,
    key: WidgetKey,
}

impl<D: ButtonDelegate + 'static> Button<D> {
    #[track_caller]
    pub fn new(colour: [f32; 4], delegate: D) -> Self {
        Button {
            colour,
            delegate,
            key: WidgetKey::caller(),
        }
    }
}

impl<D: ButtonDelegate + 'static> Description for Button<D> {
    fn key(&self) -> Option<WidgetKey> {
        Some(self.key)
    }

    fn apply(self, obj: &mut dyn Any) -> Result<(), Self> {
        match obj.downcast_mut::<ButtonWidget<D>>() {
            Some(widget) => {
                widget.colour = self.colour;
                widget.delegate = self.delegate;
                Ok(())
            }
            None => Err(self),
        }
    }

    fn create(self, cache: &mut WidgetCache) -> WidgetTree {
        cache.factory().new_widget(
            self.key,
            ButtonWidget {
                colour: self.colour,
                delegate: self.delegate,
            },
        )
    }
}

struct ButtonWidget<D: ButtonDelegate> {
    colour: [f32; 4],
    delegate: D,
}

impl<D: ButtonDelegate + 'static> Widget for ButtonWidget<D> {
    fn event(&mut self, mut state: WidgetStateMut<'_>, event: Event) {
        match event {
            Event::MousePress(_) => self.delegate.pressed(state.user_data()),
            _ => {}
        }
    }

    fn paint(&self, state: WidgetState<'_>, painter: &mut Painter) {
        painter.paint_quad(state.local_rect(), self.colour);
        painter.paint_text(wgpu_glyph::Section::new().add_text(Text::new("Hello")))
    }

    fn size_hint(&self, _: &[WidgetTree]) -> Size {
        Size::new(100.0, 100.0)
    }
}
