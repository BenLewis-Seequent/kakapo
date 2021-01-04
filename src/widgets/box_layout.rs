use std::any::Any;

use crate::description::BoxedDescription;
use crate::geom::{Position, Rect, Size};
use crate::view::{Layout, WidgetCache, WidgetKey, WidgetTree};
use crate::Description;

pub struct Box {
    widgets: Vec<BoxedDescription>,
}

impl Box {
    pub fn new() -> Box {
        Box {
            widgets: Vec::new(),
        }
    }

    pub fn append<D: Description + 'static>(mut self, desc: D) -> Self {
        self.widgets.push(BoxedDescription::new(desc));
        self
    }
}

impl Description for Box {
    fn key(&self) -> Option<WidgetKey> {
        None
    }

    fn apply(self, _: &mut dyn Any) -> Result<(), Self>
    where
        Self: Sized,
    {
        panic!("Box can't be persisted")
    }

    fn create(self, cache: &mut WidgetCache) -> WidgetTree {
        let children = self
            .widgets
            .into_iter()
            .map(|desc| cache.build(desc))
            .collect::<Vec<_>>();
        cache.factory().new_layout(BoxLayout {}, children)
    }
}

struct BoxLayout {}

impl Layout for BoxLayout {
    fn layout(&self, children: &mut [WidgetTree], _: Size) {
        let mut y = 0f32;
        for child in children {
            let size = child.size_hint();
            child.set_rect(Rect::new(Position::new(0.0, y), size));
            y += size.height;
        }
    }

    fn size_hint(&self, children: &[WidgetTree]) -> Size {
        let mut height = 0f32;
        let mut width = 0f32;
        for child in children {
            let size = child.size_hint();
            height += size.height;
            width = width.max(size.width);
        }
        Size::new(width, height)
    }
}
