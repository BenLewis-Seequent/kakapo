use std::sync::{Arc, Weak};
use crate::view::View;
use std::cell::RefCell;

struct ViewModelInner<S> {
    state: S,
    view: RefCell<Option<Arc<dyn View>>>
}

pub struct ViewModel<S> {
    inner: Arc<ViewModelInner<S>>,
}

impl<S> ViewModel<S> {
    pub fn new(state: S) -> ViewModel<S> {
        ViewModel {
            inner: Arc::new(ViewModelInner {
                state,
                view: RefCell::new(None)
            })
        }
    }

    pub fn create_ref(&self) -> ViewModelRef<S> {
        ViewModelRef {
            inner: Arc::downgrade(&self.inner)
        }
    }
}

pub struct ViewModelRef<S> {
    inner: Weak<ViewModelInner<S>>
}

pub struct ButtonViewModel {

}
