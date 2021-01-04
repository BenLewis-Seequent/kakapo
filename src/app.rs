use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use winit::platform::unix::EventLoopExtUnix;

use crate::events::EventState;
use crate::geom::{Position, Rect};
use crate::renderer::Renderer;
use crate::view::{View, ViewId, WidgetTree, WidgetTreeFactory};
use crate::view_model::ViewModel;

pub struct AppBuilder {
    windows: HashMap<winit::window::WindowId, Window>,
    event_loop: winit::event_loop::EventLoop<()>,
    app_inner: Arc<AppInner>,
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        let app_inner = Arc::new(AppInner::new());
        AppBuilder {
            windows: HashMap::new(),
            event_loop: winit::event_loop::EventLoop::<()>::new_x11_any_thread().unwrap(),
            app_inner,
        }
    }

    pub fn add_window<V: View + 'static, D: ViewModel + 'static>(&mut self, root: V, user_data: D) {
        let factory = WidgetTreeFactory {
            app: Arc::clone(&self.app_inner),
        };
        let window = Window::create(factory.new_view(root, user_data), &self.event_loop);
        self.windows.insert(window.window_id(), window);
    }

    pub fn run(self) -> ! {
        let AppBuilder {
            windows,
            event_loop,
            app_inner,
        } = self;

        let mut app = App {
            windows,
            inner: app_inner,
        };

        event_loop.run(move |event, _window_target, control_flow| {
            if winit::event::Event::MainEventsCleared == event {
                *control_flow = winit::event_loop::ControlFlow::Wait;
            }
            app.handle_winit_event(event);
            if app.windows.len() == 0 {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
        });
    }
}

pub(crate) struct AppInner {
    view_id_counter: AtomicU64,
    views_to_update: Mutex<HashSet<ViewId>>,
}

impl AppInner {
    fn new() -> AppInner {
        AppInner {
            view_id_counter: AtomicU64::new(0),
            views_to_update: Mutex::new(HashSet::new()),
        }
    }

    pub(crate) fn new_view_id(&self) -> ViewId {
        ViewId(self.view_id_counter.fetch_add(1, Ordering::Relaxed))
    }

    pub(crate) fn update_view(&self, view_id: ViewId) {
        self.views_to_update.lock().unwrap().insert(view_id);
    }
}

pub struct App {
    windows: HashMap<winit::window::WindowId, Window>,
    inner: Arc<AppInner>,
}

impl App {
    fn handle_winit_event(&mut self, winit_event: winit::event::Event<()>) {
        match winit_event {
            winit::event::Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                if window_event == winit::event::WindowEvent::Destroyed
                    || window_event == winit::event::WindowEvent::CloseRequested
                {
                    self.windows.remove(&window_id);
                    return;
                }

                let window = self.windows.get_mut(&window_id).expect("Got window");
                window.handle_event(window_event);
            }
            winit::event::Event::MainEventsCleared => {
                let views_to_update =
                    std::mem::take(self.inner.views_to_update.lock().unwrap().deref_mut());
                for window in self.windows.values_mut() {
                    if window.root.update(&views_to_update, None) {
                        let size = window
                            .window
                            .inner_size()
                            .to_logical::<f32>(window.window.scale_factor());
                        window
                            .root
                            .set_rect(Rect::new(Position::zero(), size.into()));
                    }
                    // TODO don't unconditionally redraw
                    window.window.request_redraw();
                }
            }
            winit::event::Event::RedrawRequested(window_id) => {
                let window = self.windows.get_mut(&window_id).expect("Got window");
                window.paint();
            }
            _ => {}
        }
    }
}

struct Window {
    root: WidgetTree,
    window: winit::window::Window,
    events: EventState,
    renderer: Renderer,
}

impl Window {
    fn create(
        mut root: WidgetTree,
        window_target: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Window {
        root.materialise_views(None);
        let size = root.size_hint();
        root.set_rect(Rect::new(Position::zero(), size));
        let logical_size = winit::dpi::LogicalSize::new(size.width, size.height);
        let winit_window = winit::window::WindowBuilder::new()
            .with_title("Kakapo")
            .with_inner_size(logical_size)
            .build(window_target)
            .expect("Failed to create window");

        use futures::executor::block_on;

        let renderer = block_on(Renderer::new(&winit_window));
        let events = EventState::new(&winit_window);

        Window {
            root,
            window: winit_window,
            events,
            renderer,
        }
    }

    pub(crate) fn window_id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub(crate) fn handle_event(&mut self, window_event: winit::event::WindowEvent) {
        match window_event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.renderer.resize(physical_size);
            }
            winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.renderer.resize(*new_inner_size);
            }
            event => {
                self.events.process_event(event, &mut self.root);
            }
        }
    }

    pub(crate) fn paint(&mut self) {
        let root = &mut self.root;
        match self.renderer.render(self.window.scale_factor(), |painter| {
            root.paint(None, painter)
        }) {
            Ok(_) => {}
            Err(wgpu::SwapChainError::Lost) => self.renderer.recreate(),
            Err(wgpu::SwapChainError::OutOfMemory) => panic!("swapchain: out of memory"),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
