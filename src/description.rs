use std::any::Any;

use crate::view::{WidgetCache, WidgetKey, WidgetTree};

pub trait Description {
    fn key(&self) -> Option<WidgetKey>;

    fn apply(self, obj: &mut dyn Any) -> Result<(), Self>
    where
        Self: std::marker::Sized;

    fn create(self, cache: &mut WidgetCache) -> WidgetTree
    where
        Self: std::marker::Sized;
}

trait DynDescription {
    fn key(&self) -> Option<WidgetKey>;

    fn apply(&mut self, obj: &mut dyn Any) -> bool;

    fn create(self: Box<Self>, cache: &mut WidgetCache) -> WidgetTree;
}

struct DynDescriptionImpl<D: Description> {
    // will only be None when calling apply on it's inner value
    desc: Option<D>,
}

impl<D: Description> DynDescription for DynDescriptionImpl<D> {
    fn key(&self) -> Option<WidgetKey> {
        self.desc.as_ref().unwrap().key()
    }

    fn apply(&mut self, obj: &mut dyn Any) -> bool {
        match self.desc.take().unwrap().apply(obj) {
            Ok(()) => true,
            Err(desc) => {
                self.desc = Some(desc);
                false
            }
        }
    }

    fn create(self: Box<Self>, cache: &mut WidgetCache) -> WidgetTree {
        self.desc.unwrap().create(cache)
    }
}

pub struct BoxedDescription {
    inner: Box<dyn DynDescription>,
}

impl BoxedDescription {
    pub fn new<D: Description + 'static>(desc: D) -> BoxedDescription {
        BoxedDescription {
            inner: Box::new(DynDescriptionImpl { desc: Some(desc) }),
        }
    }
}

impl Description for BoxedDescription {
    fn key(&self) -> Option<WidgetKey> {
        self.inner.key()
    }

    fn apply(mut self, obj: &mut dyn Any) -> Result<(), Self> {
        if self.inner.apply(obj) {
            Ok(())
        } else {
            Err(self)
        }
    }

    fn create(self, cache: &mut WidgetCache) -> WidgetTree {
        self.inner.create(cache)
    }
}
