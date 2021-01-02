use std::any::Any;

use crate::renderer::painter::Painter;
use crate::geom::{Size, Rect, Position};
use crate::events::Event;


struct ViewData<V: View + ?Sized> {
    widget: Option<WidgetTree>,
    user_data: Option<Box<dyn Any>>,
    view: V
}

pub type UserData<'a> = Option<&'a dyn Any>;
pub type UserDataMut<'a> = Option<&'a mut dyn Any>;

pub struct WidgetState<'a> {
    rect: Rect,
    user_data: UserData<'a>,
}

impl<'a> WidgetState<'a> {
    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn local_rect(&self) -> Rect {
        let rect = self.rect();
        Rect::new(Position::zero(), rect.size)
    }

    pub fn user_data(&self) -> UserData<'_> {
        match self.user_data {
            Some(ref data) => Some(*data),
            None => None
        }
    }
}

pub struct WidgetStateMut<'a> {
    rect: Rect,
    user_data: UserDataMut<'a>,
}

impl<'a> WidgetStateMut<'a> {
    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn local_rect(&self) -> Rect {
        let rect = self.rect();
        Rect::new(Position::zero(), rect.size)
    }

    pub fn user_data(&mut self) -> UserDataMut<'_> {
        match self.user_data {
            Some(ref mut data) => Some(*data),
            None => None
        }
    }
}


struct WidgetData<W: Widget + ?Sized> {
    allocation: Option<Rect>,
    children: Vec<WidgetTree>,
    widget: W,
}

struct LayoutData<L: Layout + ?Sized> {
    allocation: Option<Rect>,
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

    pub fn new_view<V: View + 'static, D: 'static>(view: V, user_data: D) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::View(Box::new(ViewData {
            widget: None,
            user_data: Some(Box::new(user_data) as Box<dyn Any>),
            view
        }) as Box<ViewData<dyn View>>))
    }

    pub fn new_widget<W: Widget + 'static>(widget: W) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::Widget(Box::new(WidgetData {
            allocation: None,
            children: Vec::new(),
            widget
        }) as Box<WidgetData<dyn Widget>>))
    }

    pub fn new_layout<L: Layout + 'static>(layout: L, children: Vec<WidgetTree>) -> WidgetTree {
        WidgetTree::new(WidgetTreeInner::Layout(Box::new(LayoutData {
            allocation: None,
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
                for child in w.children.iter_mut() {
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

    fn find_widget_at(&mut self,
                      pos: Position,
                      user_data: UserDataMut<'_>,
                      func: impl FnOnce(&mut WidgetData<dyn Widget>, UserDataMut<'_>, Position)) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if let Some(w) = &mut view.widget {
                    w.find_widget_at(pos, view.user_data.as_deref_mut().or(user_data), func)
                } else {
                    panic!("View widget is None when processing event");
                }
            }
            WidgetTreeInner::Widget(ref mut w) => {
                // TODO child widgets
                let rect = w.allocation.unwrap();
                if rect.contains(pos) {
                    func(w, user_data, pos - rect.origin);
                }
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.iter_mut() {
                    if child.rect().contains(pos) {
                        child.find_widget_at(pos, user_data, func);
                        break;
                    }
                }
            }
        }
    }

    fn position_event(&mut self, pos: Position, event_constructor: impl FnOnce(Position) -> Event) {
        self.find_widget_at(pos, None, |w, user_data, pos| {
            let state = WidgetStateMut {
                rect: w.allocation.unwrap(),
                user_data
            };
            w.widget.event(state, event_constructor(pos));
        });
    }

    pub(crate) fn event(&mut self, event: Event) {
        match event {
            Event::MousePress(pos) => self.position_event(pos, |pos| Event::MousePress(pos)),
            Event::MouseRelease(pos) => self.position_event(pos, |pos| Event::MouseRelease(pos)),
        }
    }

    pub(crate) fn paint(&mut self, user_data: UserData<'_>, painter: &mut Painter<'_>) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if let Some(w) = &mut view.widget {
                    w.paint(view.user_data.as_deref().or(user_data), painter)
                } else {
                    panic!("View widget is None when painting");
                }
            }
            WidgetTreeInner::Widget(ref w) => {
                let state = WidgetState {
                    rect: w.allocation.unwrap(),
                    user_data
                };
                w.widget.paint(state, &mut painter.with_rect(w.allocation.unwrap()));
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.iter_mut() {
                    child.paint(user_data, painter);
                }
            }
        }
    }

    fn rect(&self) -> Rect {
        match self.inner {
            WidgetTreeInner::View(ref view) => {
                if let Some(w) = &view.widget {
                    w.rect()
                } else {
                    panic!("View widget is None when querying rect");
                }
            }
            WidgetTreeInner::Widget(ref w) => {
                w.allocation.unwrap()
            },
            WidgetTreeInner::Layout(ref layout) => {
                layout.allocation.unwrap()
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
                w.allocation = Some(rect);
            },
            WidgetTreeInner::Layout(ref mut layout) => {
                layout.allocation = Some(rect);
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
                w.widget.size_hint(w.children.as_slice())
            },
            WidgetTreeInner::Layout(ref layout) => {
                layout.layout.size_hint(layout.children.as_slice())
            }
        }
    }
}

pub trait Widget {
    fn event(&mut self, state: WidgetStateMut<'_>, event: Event);

    fn paint(&self, state: WidgetState<'_>, painter: &mut Painter);

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
