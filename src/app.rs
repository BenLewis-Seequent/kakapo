use std::collections::HashMap;

use crate::view::{View, WidgetTree};
use crate::renderer::Renderer;
use winit::platform::unix::EventLoopExtUnix;
use crate::geom::{Rect, Position};
use crate::events::EventState;


pub struct AppBuilder {
    windows: HashMap<winit::window::WindowId, Window>,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder {
            windows: HashMap::new(),
            event_loop: winit::event_loop::EventLoop::<()>::new_x11_any_thread().unwrap(),
        }
    }

    pub fn add_window<V: View + 'static>(&mut self, root: V) {
        let window = Window::create(WidgetTree::new_view(root),
                                    &self.event_loop);
        self.windows.insert(window.window_id(), window);
    }

    pub fn run(self) -> ! {
        let AppBuilder {
            windows,
            event_loop,
        } = self;

        let mut app = App { windows };

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

pub struct App {
    windows: HashMap<winit::window::WindowId, Window>,
}

impl App {
    fn handle_winit_event(&mut self, winit_event: winit::event::Event<()>) {
        match winit_event {
            winit::event::Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                if window_event == winit::event::WindowEvent::Destroyed || window_event == winit::event::WindowEvent::CloseRequested {
                    self.windows.remove(&window_id);
                    return;
                }

                let window = self.windows.get_mut(&window_id).expect("Got window");
                window.handle_event(window_event);
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
        window_target: &winit::event_loop::EventLoopWindowTarget<()>
    ) -> Window {
        root.materialise_views();
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
            renderer
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
        match self.renderer.render(self.window.scale_factor(), |painter| root.paint(painter)) {
            Ok(_) => {}
            Err(wgpu::SwapChainError::Lost) => self.renderer.recreate(),
            Err(wgpu::SwapChainError::OutOfMemory) => panic!("swapchain: out of memory"),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}


