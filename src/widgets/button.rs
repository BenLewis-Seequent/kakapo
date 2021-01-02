use std::any::Any;

use crate::view::{Description, WidgetTree, Widget, WidgetCache, WidgetState};
use crate::renderer::painter::Painter;
use crate::geom::Size;
use crate::events::Event;

pub struct Button {
    colour: [f32; 4]
}

impl Button {
    pub fn new(colour: [f32; 4]) -> Button {
        Button {
            colour
        }
    }
}

impl Description for Button {
    fn apply(&self, obj: &mut dyn Any) {

    }

    fn create(&self, _: &mut WidgetCache) -> WidgetTree {
        WidgetTree::new_widget(ButtonWidget { colour: self.colour })
    }
}


struct ButtonWidget {
    colour: [f32; 4]
}

impl Widget for ButtonWidget {
    fn event(&mut self, state: &mut WidgetState, event: Event) {
    }

    fn paint(&self, state: &WidgetState, painter: &mut Painter) {
        painter.paint_quad(state.local_rect(), self.colour);
    }

    fn size_hint(&self, children: &[WidgetTree]) -> Size {
        Size::new(100.0, 100.0)
    }
}
