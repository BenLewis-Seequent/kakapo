use kakapo::app;
use kakapo::view_model;
use kakapo::view;
use kakapo::view::{View, WidgetTree, WidgetCache};
use kakapo::widgets;
use std::sync::Arc;
use kakapo::view_model::ViewModelRef;


pub struct ButtonViewModel {

}

pub struct ButtonView {
    pub view_model: ViewModelRef<ButtonViewModel>
}

impl View for ButtonView {
    fn view(&mut self, cache: &mut WidgetCache) -> WidgetTree {
        cache.build( widgets::Box::new()
            .append(widgets::Button::new([1.0, 0.0, 0.0, 1.0]))
            .append(widgets::Button::new([0.0, 1.0, 0.0, 1.0])))
    }
}


fn main() {
    let vm = view_model::ViewModel::new(ButtonViewModel {});

    let mut app = app::AppBuilder::new();
    app.add_window(ButtonView {
        view_model: vm.create_ref()
    });

    app.run()
}
