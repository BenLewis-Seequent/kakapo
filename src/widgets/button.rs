use std::any::Any;

use crate::view::{Description, WidgetTree, Widget, WidgetCache};
use crate::renderer::painter::Painter;
use crate::geom::{Rect, Position, Size};

pub struct Button {

}

impl Button {
    pub fn new() -> Button {
        Button {}
    }
}

impl Description for Button {
    fn apply(&self, obj: &mut dyn Any) {

    }

    fn create(&self, _: &mut WidgetCache) -> WidgetTree {
        WidgetTree::new_widget(ButtonWidget {})
    }
}


struct ButtonWidget {

}

impl Widget for ButtonWidget {
    fn paint(&self, painter: &mut Painter) {
        painter.paint_quad(Rect::new(Position::zero(), Size::new(1.0, 1.0)), [1.0, 1.0, 0.0, 1.0]);
    }

    fn size_hint(&self, children: &[WidgetTree]) -> Size {
        Size::new(100.0, 100.0)
    }
}
