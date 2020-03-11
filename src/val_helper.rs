use crate::data::Val;

pub trait Get {
  fn clone_as<T: 'static + Clone>(&self) -> Option<T>;
  fn copy_as<T: 'static + Copy>(&self) -> Option<T>;
}

impl Get for Val {
  fn clone_as<T: 'static + Clone>(&self) -> Option<T> {
    self.downcast_ref::<T>().cloned()
  }

  fn copy_as<T: 'static + Copy>(&self) -> Option<T> {
    self.downcast_ref::<T>().copied()
  }
}

#[test]
fn test() {
  use crate::data::r;
  assert_eq!(r(123i32).copy_as::<i32>(), Some(123i32));
  assert_eq!(r(Box::new(123i32)).clone_as::<Box<i32>>(), Some(Box::new(123i32)));
  assert_eq!(r(123i32).copy_as::<i64>(), None);
  assert_eq!(r(Box::new(123i32)).clone_as::<Box<i64>>(), None);
}
