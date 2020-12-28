use crate::view_model::ViewModel;

mod view_model;
mod view;
mod app;
mod geom;
mod renderer;


fn main() {
    let vm = ViewModel::new(view_model::ButtonViewModel {});

    let mut app = app::AppBuilder::new();
    app.add_window(view::ButtonView {
        view_model: vm.create_ref()
    });

    app.run()
}
