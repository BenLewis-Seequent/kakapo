use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::app::AppInner;
use crate::description::Description;
use crate::events::Event;
use crate::geom::{Position, Rect, Size};
use crate::renderer::painter::Painter;
use crate::view_model::ViewModel;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct ViewId(pub(crate) u64);

struct ViewData<V: View + ?Sized> {
    view_id: ViewId,
    widget: Option<WidgetTree>,
    user_data: Option<Box<dyn Any>>,
    view: V,
}

pub type UserData<'a> = Option<&'a dyn Any>;
pub type UserDataMut<'a> = Option<&'a mut dyn Any>;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
enum WidgetKeyInner {
    Location(&'static std::panic::Location<'static>),
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct WidgetKey {
    inner: WidgetKeyInner,
}

impl WidgetKey {
    #[track_caller]
    pub fn caller() -> WidgetKey {
        WidgetKey {
            inner: WidgetKeyInner::Location(std::panic::Location::caller()),
        }
    }
}

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
            None => None,
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
            None => None,
        }
    }
}

struct WidgetData<W: Widget + ?Sized> {
    key: WidgetKey,
    allocation: Option<Rect>,
    children: Vec<WidgetTree>,
    widget: W,
}

struct LayoutData<L: Layout + ?Sized> {
    allocation: Option<Rect>,
    children: Vec<WidgetTree>,
    layout: L,
}

enum WidgetTreeInner {
    View(Box<ViewData<dyn View>>),
    Widget(Box<WidgetData<dyn Widget>>),
    Layout(Box<LayoutData<dyn Layout>>),
}

pub struct WidgetTreeFactory {
    pub(crate) app: Arc<AppInner>,
}

impl WidgetTreeFactory {
    fn new(&self, inner: WidgetTreeInner) -> WidgetTree {
        WidgetTree {
            app: Arc::clone(&self.app),
            inner,
        }
    }

    pub fn new_view<V: View + 'static, D: ViewModel + 'static>(
        &self,
        view: V,
        user_data: D,
    ) -> WidgetTree {
        let view_id = self.app.new_view_id();
        user_data.view_refs().init(Arc::clone(&self.app), view_id);

        self.new(WidgetTreeInner::View(Box::new(ViewData {
            view_id,
            widget: None,
            user_data: Some(Box::new(user_data) as Box<dyn Any>),
            view,
        }) as Box<ViewData<dyn View>>))
    }

    pub fn new_widget<W: Widget + 'static>(&self, key: WidgetKey, widget: W) -> WidgetTree {
        self.new(WidgetTreeInner::Widget(Box::new(WidgetData {
            key,
            allocation: None,
            children: Vec::new(),
            widget,
        })
            as Box<WidgetData<dyn Widget>>))
    }

    pub fn new_layout<L: Layout + 'static>(
        &self,
        layout: L,
        children: Vec<WidgetTree>,
    ) -> WidgetTree {
        self.new(WidgetTreeInner::Layout(Box::new(LayoutData {
            allocation: None,
            children,
            layout,
        })
            as Box<LayoutData<dyn Layout>>))
    }
}

pub struct WidgetTree {
    app: Arc<AppInner>,
    inner: WidgetTreeInner,
}

impl WidgetTree {
    pub(crate) fn materialise_views(&mut self, user_data: UserData<'_>) {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                if view.widget.is_none() {
                    let user_data = view.user_data.as_deref().or(user_data);
                    let mut cache = WidgetCache {
                        factory: WidgetTreeFactory {
                            app: Arc::clone(&self.app),
                        },
                        cached: HashMap::new(),
                    };
                    let mut tree = view.view.view(&mut cache, user_data);
                    tree.materialise_views(user_data);
                    view.widget = Some(tree);
                }
            }
            WidgetTreeInner::Widget(ref mut w) => {
                for child in w.children.iter_mut() {
                    child.materialise_views(user_data);
                }
            }
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.iter_mut() {
                    child.materialise_views(user_data);
                }
            }
        }
    }

    fn deconstruct(mut self, widgets: &mut HashMap<WidgetKey, WidgetTree>) {
        match self.inner {
            WidgetTreeInner::View(_) => {}
            WidgetTreeInner::Widget(ref mut w) => {
                for child in w.children.drain(..) {
                    child.deconstruct(widgets);
                }
                widgets.insert(w.key, self);
            }
            WidgetTreeInner::Layout(ref mut layout) => {
                for child in layout.children.drain(..) {
                    child.deconstruct(widgets);
                }
            }
        }
    }

    fn find_widget_at(
        &mut self,
        pos: Position,
        user_data: UserDataMut<'_>,
        func: impl FnOnce(&mut WidgetData<dyn Widget>, UserDataMut<'_>, Position),
    ) {
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
            }
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
                user_data,
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

    pub(crate) fn update(&mut self, views: &HashSet<ViewId>, user_data: UserData<'_>) -> bool {
        match self.inner {
            WidgetTreeInner::View(ref mut view) => {
                let user_data = view.user_data.as_deref().or(user_data);
                if views.contains(&view.view_id) {
                    // TODO child widgets{
                    let mut cached = HashMap::new();
                    view.widget.take().map(|tree| tree.deconstruct(&mut cached));
                    let mut cache = WidgetCache {
                        factory: WidgetTreeFactory {
                            app: Arc::clone(&self.app),
                        },
                        cached,
                    };
                    let mut tree = view.view.view(&mut cache, user_data);
                    tree.update(views, user_data);
                    view.widget = Some(tree);
                    true
                } else {
                    if let Some(w) = &mut view.widget {
                        w.update(views, user_data)
                    } else {
                        panic!("View widget is None when updating views");
                    }
                }
            }
            WidgetTreeInner::Widget(_) => {
                // TODO child widgets
                false
            }
            WidgetTreeInner::Layout(ref mut layout) => {
                // TODO determine whether the update needs to bubble up, i.e. the size hints are the
                //  same then the parent doesn't need to re-layout.
                let mut updated = false;
                for child in layout.children.iter_mut() {
                    if child.update(views, user_data) {
                        updated = true
                    }
                }
                updated
            }
        }
    }

    pub(crate) fn paint(&self, user_data: UserData<'_>, painter: &mut Painter<'_>) {
        match self.inner {
            WidgetTreeInner::View(ref view) => {
                if let Some(w) = &view.widget {
                    w.paint(view.user_data.as_deref().or(user_data), painter)
                } else {
                    panic!("View widget is None when painting");
                }
            }
            WidgetTreeInner::Widget(ref w) => {
                let state = WidgetState {
                    rect: w.allocation.unwrap(),
                    user_data,
                };
                w.widget
                    .paint(state, &mut painter.with_rect(w.allocation.unwrap()));
            }
            WidgetTreeInner::Layout(ref layout) => {
                for child in layout.children.iter() {
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
            WidgetTreeInner::Widget(ref w) => w.allocation.unwrap(),
            WidgetTreeInner::Layout(ref layout) => layout.allocation.unwrap(),
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
            }
            WidgetTreeInner::Layout(ref mut layout) => {
                layout.allocation = Some(rect);
                // TODO pass rect origin, probably encapsulate it in a layout context
                layout
                    .layout
                    .layout(layout.children.as_mut_slice(), rect.size);
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
            WidgetTreeInner::Widget(ref w) => w.widget.size_hint(w.children.as_slice()),
            WidgetTreeInner::Layout(ref layout) => {
                layout.layout.size_hint(layout.children.as_slice())
            }
        }
    }

    fn obj_mut(&mut self) -> &mut dyn Any {
        match self.inner {
            WidgetTreeInner::View(_) => panic!(),
            WidgetTreeInner::Widget(ref mut w) => w.widget.as_any_mut(),
            WidgetTreeInner::Layout(_) => panic!(),
        }
    }
}

pub trait Downcast {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> Downcast for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Widget: Downcast {
    fn event(&mut self, state: WidgetStateMut<'_>, event: Event);

    fn paint(&self, state: WidgetState<'_>, painter: &mut Painter);

    fn size_hint(&self, children: &[WidgetTree]) -> Size;
}

pub trait Layout {
    fn layout(&self, children: &mut [WidgetTree], size: Size);

    fn size_hint(&self, children: &[WidgetTree]) -> Size;
}

pub struct WidgetCache {
    factory: WidgetTreeFactory,
    cached: HashMap<WidgetKey, WidgetTree>,
}

impl WidgetCache {
    pub fn build<D: Description>(&mut self, desc: D) -> WidgetTree {
        match desc.key().and_then(|key| self.cached.remove(&key)) {
            Some(mut widget) => match desc.apply(widget.obj_mut()) {
                Ok(()) => widget,
                Err(desc) => desc.create(self),
            },
            None => desc.create(self),
        }
    }

    pub fn factory(&self) -> &WidgetTreeFactory {
        &self.factory
    }
}

pub trait View {
    fn view(&mut self, cache: &mut WidgetCache, user_data: UserData<'_>) -> WidgetTree;
}

pub struct ViewRefs {
    inner: Mutex<Vec<(Arc<AppInner>, ViewId)>>,
}

impl ViewRefs {
    pub fn new() -> ViewRefs {
        ViewRefs {
            inner: Mutex::new(Vec::new()),
        }
    }

    fn init(&self, app: Arc<AppInner>, view_id: ViewId) {
        self.inner.lock().unwrap().push((app, view_id))
    }

    pub fn update(&self) {
        let guard = self.inner.lock().unwrap();
        for (app, view_id) in guard.iter() {
            app.update_view(*view_id);
        }
    }
}
