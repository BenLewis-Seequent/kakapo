use std::any::Any;

use crate::view::{Description, WidgetTree, Widget, WidgetCache, WidgetState, WidgetStateMut, UserDataMut};
use crate::renderer::painter::Painter;
use crate::geom::Size;
use crate::events::Event;

// TODO remove copy bound
pub trait ButtonDelegate : Copy {
    fn pressed(&mut self, parent: UserDataMut<'_>);
}

pub struct Button<D: ButtonDelegate + 'static> {
    colour: [f32; 4],
    delegate: D
}

impl<D: ButtonDelegate + 'static> Button<D> {
    pub fn new(colour: [f32; 4], delegate: D) -> Self {
        Button {
            colour,
            delegate
        }
    }
}

impl<D: ButtonDelegate + 'static> Description for Button<D> {
    fn apply(&self, obj: &mut dyn Any) {

    }

    fn create(&self, _: &mut WidgetCache) -> WidgetTree {
        WidgetTree::new_widget(ButtonWidget {
            colour: self.colour,
            delegate: self.delegate
        })
    }
}


struct ButtonWidget<D: ButtonDelegate> {
    colour: [f32; 4],
    delegate: D
}

impl<D: ButtonDelegate> Widget for ButtonWidget<D> {
    fn event(&mut self, mut state: WidgetStateMut<'_>, event: Event) {
        match event {
            Event::MousePress(_) => {
                self.delegate.pressed(state.user_data())
            }
            _ => {}
        }
    }

    fn paint(&self, state: WidgetState<'_>, painter: &mut Painter) {
        painter.paint_quad(state.local_rect(), self.colour);
    }

    fn size_hint(&self, children: &[WidgetTree]) -> Size {
        Size::new(100.0, 100.0)
    }
}
