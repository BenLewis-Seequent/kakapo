use crate::view::{Description, WidgetTree, Layout, WidgetCache};
use std::any::Any;
use crate::geom::{Size, Rect, Position};

pub struct Box {
    widgets: Vec<std::boxed::Box<dyn Description>>
}

impl Box {
    pub fn new() -> Box {
        Box {
            widgets: Vec::new(),
        }
    }

    pub fn append<D: Description + 'static>(&mut self, desc: D) -> &mut Self {
        self.widgets.push(std::boxed::Box::new(desc));
        self
    }
}

impl Description for Box {
    fn apply(&self, obj: &mut dyn Any) {

    }

    fn create(&self, cache: &mut WidgetCache) -> WidgetTree {
        let children = self.widgets.iter().map(|desc| cache.build(desc.as_ref())).collect::<Vec<_>>();
        WidgetTree::new_layout(BoxLayout {}, children)
    }
}

struct BoxLayout {}

impl Layout for BoxLayout {
    fn layout(&self, children: &mut [WidgetTree], size: Size) {
        let mut y = 0f32;
        for child in children {
            let size = child.size_hint();
            child.set_rect(Rect::new(Position::new(0.0, y), size));
            y += size.height;
        }
    }

    fn size_hint(&self, children: &[WidgetTree]) -> Size {
        let mut size = Size::zero();
        for child in children {
            size += child.size_hint();
        }
        size
    }
}




