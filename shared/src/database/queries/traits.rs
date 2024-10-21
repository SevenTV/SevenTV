#[doc(hidden)]
pub(crate) trait Sealed {}

#[doc(hidden)]
pub trait __ArrayLike<T> {}

impl<T: IntoIterator<Item = U>, U> __ArrayLike<U> for T {}
