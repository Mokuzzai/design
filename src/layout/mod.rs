pub mod primitive;
pub mod composite;

pub use primitive::NonZst;
pub use primitive::TypeLayout;
pub use primitive::Zst;
pub use std::alloc::Layout;

pub use composite::Array;
pub use composite::Multi;
pub use composite::Slice;
pub use composite::Struct;

use std::alloc::LayoutError;
use std::convert::Infallible;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub type Result<T, E = LayoutError> = std::result::Result<T, E>;


#[derive(Debug)]
pub struct Alignment(NonZeroUsize); // NOTE: use `std::ptr::Alignment`?

impl Alignment {
	fn get(&self) -> usize {
		self.0.get()
	}
}

/// # Safety
/// * [`ToLayout::to_layout`] must be a pure function, for the following to be safe:
/// ```
/// # use design::layout::TypeLayout;
/// # use design::layout::ToLayout;
///
/// let layout = TypeLayout::<u32>::new();
///
/// let a = layout.to_layout();
/// let b = layout.to_layout();
///
/// # assert_eq!(a, b);
///
/// let ptr = unsafe { std::alloc::alloc(a) };
///
/// unsafe { std::alloc::dealloc(ptr, b) };
/// ```
pub unsafe trait ToLayout {
	fn to_layout(&self) -> Layout;
}

unsafe impl ToLayout for Layout {
	fn to_layout(&self) -> Layout {
		*self
	}
}

/// # Safety
/// See [`ToLayout`]: Safety
pub unsafe trait TryToLayout {
	type Error;

	fn try_to_layout(&self) -> Result<Layout, Self::Error>;
}

unsafe impl<L> TryToLayout for L where L: ToLayout {
	type Error = Infallible;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		Ok(self.to_layout())
	}
}
