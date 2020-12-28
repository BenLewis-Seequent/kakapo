use crate::view_model::{ViewModelRef, ButtonViewModel};
use crate::renderer::painter::Painter;
use crate::geom::{Size, Rect, Position};


pub struct ViewData<V: View + ?Sized> {
    widget: Option<WidgetTree>,
    view: V
}

pub struct WidgetData<W: Widget + ?Sized> {
    size: Size,
    widget: W,
}

pub enum WidgetTree {
    View(Box<ViewData<dyn View>>),
    Widget(Box<WidgetData<dyn Widget>>)
}

impl WidgetTree {
    pub fn new_view<V: View + 'static>(view: V) -> WidgetTree {
        WidgetTree::View(Box::new(ViewData {
            widget: None,
            view
        }) as Box<ViewData<dyn View>>)
    }

    pub(super) fn paint(&mut self, painter: &mut Painter<'_>) {
        match self {
            WidgetTree::View(view) => {
                if let Some(w) = &mut view.widget {
                    w.paint(painter)
                } else {
                    // Probably should just panic
                    let mut tree = view.view.view();
                    tree.paint(painter);
                    view.widget = Some(tree);
                }
            }
            WidgetTree::Widget(w) => {
                w.widget.paint(painter);
            }
        }
    }
}

pub trait Widget {
    fn paint(&self, painter: &mut Painter);
}

pub trait View {
    fn view(&mut self) -> WidgetTree;
}

pub struct ButtonView {
    pub view_model: ViewModelRef<ButtonViewModel>
}

impl View for ButtonView {
    fn view(&mut self) -> WidgetTree {
        WidgetTree::Widget(Box::new(WidgetData {
            size: Size::new(100.0, 100.0),
            widget: Button {}
        }))
    }
}

struct Button {

}

impl Widget for Button {
    fn paint(&self, painter: &mut Painter) {
        painter.paint_quad(Rect::new(Position::zero(), Size::new(1.0, 1.0)), [1.0, 1.0, 0.0, 1.0]);
    }
}
