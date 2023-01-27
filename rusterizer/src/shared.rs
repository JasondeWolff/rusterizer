pub use std::rc::Rc;
pub use std::cell::{RefCell, Ref, RefMut};

#[derive(Debug, Clone)]
pub struct Shared<T> {
    value: Option<Rc<RefCell<T>>>
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared {
            value: Some(Rc::new(RefCell::new(value)))
        }
    }

    pub fn empty() -> Self {
        Shared {
            value: None
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.value {
            Some(_) => false,
            None => true
        }
    }

    pub fn strong_count(&self) -> usize {
        Rc::strong_count(self.value.as_ref().unwrap())
    }

    pub fn as_ref(&self) -> Ref<'_, T> {
        self.value.as_ref().unwrap().as_ref().borrow()
    }

    pub fn as_mut(&self) -> RefMut<'_, T> {
        self.value.as_ref().unwrap().as_ref().borrow_mut()
    }

    pub fn as_ptr(&self) -> *const T {
        RefCell::as_ptr(self.value.as_ref().unwrap())
    }

    pub fn try_as_ref(&self) -> Option<Ref<'_, T>> {
        match self.value.as_ref() {
            Some(value) => Some(value.as_ref().borrow()),
            None => None
        }
    }

    pub fn try_as_mut(&self) -> Option<RefMut<'_, T>> {
        match self.value.as_ref() {
            Some(value) => Some(value.as_ref().borrow_mut()),
            None => None
        }
    }
}