use std::any::Any;

use crate::renderer::painter::Painter;
use crate::geom::{Size, Rect, Position};


struct ViewData<V: View + ?Sized> {
    widget: Option<WidgetTree>,
    view: V
}

pub struct WidgetState {
    allocation: Option<Rect>,
    children: Vec<WidgetTree>
}

impl WidgetState {
    pub fn rect(&self) -> Rect {
        self.allocation.unwrap()
    }

    pub fn local_rect(&self) -> Rect {
        let rect = self.rect();
        Rect::new(Position::zero(), rect.size)
    }
}

struct WidgetData<W: Widget + ?Sized> {
    state: WidgetState,
    widget: W,
}

struct LayoutData<L: Layout + ?Sized> {
    children: Vec<WidgetTree>,
    layout: L
}

enum WidgetTreeInner {
    View(Box<ViewData<dyn View>>),
    Widget(Box<WidgetData<dyn Widget>>),
    Layout(Box<LayoutData<dyn Layout>>),
}

pub struct WidgetTree {
    inner: WidgetTreeInner
}

impl WidgetTree {
    fn new(inner: WidgetTreeInner) -> WidgetTree {
        WidgetTree {
            inner
        }
    }

    pub fn new_view<V: View + 'static>(view: V) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::View(Box::new(ViewData {
            widget: None,
            view
        }) as Box<ViewData<dyn View>>))
    }

    pub fn new_widget<W: Widget + 'static>(widget: W) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::Widget(Box::new(WidgetData {
            state: WidgetState {
                allocation: None,
                children: Vec::new(),
            },
            widget
        }) as Box<WidgetData<dyn Widget>>))
    }

    pub fn new_layout<L: Layout + 'static>(layout: L, children: Vec<WidgetTree>) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::Layout(Box::new(LayoutData {
            children,
            layout
        }) as Box<LayoutData<dyn Layout>>))
    }

    pub(crate) fn materialise_views(&mut self) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if view.widget.is_none() {
                    let mut tree = view.view.view(&mut WidgetCache {});
                    tree.materialise_views();
                    view.widget = Some(tree);
                }
            }
            WidgetTreeInner::Widget(ref mut w) => {
                for child in w.state.children.iter_mut() {
                    child.materialise_views();
                }
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.iter_mut() {
                    child.materialise_views();
                }
            }
        }
    }

    pub(super) fn paint(&mut self, painter: &mut Painter<'_>) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if let Some(w) = &mut view.widget {
                    w.paint(painter)
                } else {
                    panic!("View widget is None when painting");
                }
            }
            WidgetTreeInner::Widget(ref w) => {
                w.widget.paint(&w.state, &mut painter.with_rect(w.state.rect()));
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.iter_mut() {
                    child.paint(painter);
                }
            }
        }
    }

    // TODO should only called during layout
    pub fn set_rect(&mut self, rect: Rect) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if let Some(w) = &mut view.widget {
                    w.set_rect(rect);
                } else {
                    panic!("View widget is None when setting rect");
                }
            }
            WidgetTreeInner::Widget(ref mut w) => {
                w.state.allocation = Some(rect);
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                // TODO pass rect origin, probably encapsulate it in a layout context
                layout.layout.layout(layout.children.as_mut_slice(), rect.size);
            }
        }
    }

    // TODO return min/max size aksi
    pub fn size_hint(&self) -> Size {
        match self.inner {
            WidgetTreeInner::View(ref view) => {
                if let Some(w) = &view.widget {
                    w.size_hint()
                } else {
                    panic!("View widget is None when querying size hint");
                }
            }
            WidgetTreeInner::Widget(ref w) => {
                w.widget.size_hint(w.state.children.as_slice())
            },
            WidgetTreeInner::Layout(ref layout) => {
                layout.layout.size_hint(layout.children.as_slice())
            }
        }
    }
}

pub trait Widget {
    fn paint(&self, state: &WidgetState, painter: &mut Painter);

    fn size_hint(&self, children: &[WidgetTree]) -> Size;
}

pub trait Layout {
    fn layout(&self, children: &mut [WidgetTree], size: Size);

    fn size_hint(&self, children: &[WidgetTree]) -> Size;
}

pub trait Description {
    fn apply(&self, obj: &mut dyn Any);

    fn create(&self, cache: &mut WidgetCache) -> WidgetTree;
}

pub struct WidgetCache {

}

impl WidgetCache {
    pub fn build<D: Description + ?Sized>(&mut self, desc: &D) -> WidgetTree {
        desc.create(self)
    }
}

pub trait View {
    fn view(&mut self, cache: &mut WidgetCache) -> WidgetTree;
}
