use crate::view::ViewRefs;


pub trait ViewModel {
    fn view_refs(&self) -> &ViewRefs;
}
