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
        let mut b = widgets::Box::new();
        b.append(widgets::Button::new());
        b.append(widgets::Button::new());
        cache.build(&b)
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
