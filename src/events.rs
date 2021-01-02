use winit::dpi::PhysicalPosition;
use crate::geom::Position;
use crate::view::WidgetTree;

#[derive(Copy, Clone)]
pub enum Event {
    MousePress(Position),
    MouseRelease(Position),
}

pub(crate) struct EventState {
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    current_modifiers: winit::event::ModifiersState,
    scale_factor: f64,
}

impl EventState {
    pub(crate) fn new(window: &winit::window::Window) -> EventState {
        EventState {
            cursor_position: PhysicalPosition::new(0.0, 0.0),
            current_modifiers: winit::event::ModifiersState::default(),
            scale_factor: window.scale_factor(),
        }
    }

    fn cursor_logical_position(&self) -> Position {
        let logical = self.cursor_position.to_logical(self.scale_factor);
        Position::new(logical.x, logical.y)
    }

    pub(crate) fn process_event(
        &mut self,
        window_event: winit::event::WindowEvent<'_>,
        root: &mut WidgetTree,
    ) {
        match window_event {
            winit::event::WindowEvent::ModifiersChanged(state) => {
                self.current_modifiers = state;
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = position;
            }
            winit::event::WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } => {
                root.event(Event::MousePress(self.cursor_logical_position()));
            }
            winit::event::WindowEvent::MouseInput {
                state: winit::event::ElementState::Released,
                ..
            } => {
                root.event(Event::MouseRelease(self.cursor_logical_position()));
            }
            _ => {}
        }
    }
}

