use kakapo::app;
use kakapo::view_model;
use kakapo::view;
use kakapo::view::{View, WidgetTree, WidgetCache, UserDataMut};
use kakapo::widgets;
use std::sync::Arc;
use kakapo::view_model::ViewModelRef;
use kakapo::widgets::ButtonDelegate;

struct AppData {
    two_buttons: bool
}

impl AppData {
    fn new() -> AppData {
        AppData {
            two_buttons: false,
        }
    }
}

#[derive(Copy, Clone)]
struct PrimaryButtonDelegate;

impl ButtonDelegate for PrimaryButtonDelegate {
    fn pressed(&mut self, parent: UserDataMut<'_>) {
        let app_data = parent.unwrap().downcast_mut::<AppData>().unwrap();
        app_data.two_buttons = !app_data.two_buttons;
        println!("First");
    }
}

#[derive(Copy, Clone)]
struct SecondaryButtonDelegate;

impl ButtonDelegate for SecondaryButtonDelegate {
    fn pressed(&mut self, parent: UserDataMut<'_>) {
        println!("Second");
    }
}


struct AppView { }

impl View for AppView {
    fn view(&mut self, cache: &mut WidgetCache) -> WidgetTree {
        cache.build( widgets::Box::new()
            .append(widgets::Button::new([1.0, 0.0, 0.0, 1.0], PrimaryButtonDelegate))
            .append(widgets::Button::new([0.0, 1.0, 0.0, 1.0], SecondaryButtonDelegate)))
    }
}


fn main() {
    let mut app = app::AppBuilder::new();
    app.add_window(AppView {}, AppData::new());
    app.run()
}
