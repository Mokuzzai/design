use super::*;

pub struct TypeLayout<T>(PhantomData<T>);

impl<T> TypeLayout<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

unsafe impl<T> ToLayout for TypeLayout<T> {
	fn to_layout(&self) -> Layout {
		Layout::new::<T>()
	}
}

#[derive(Debug)]
pub struct NonZst {
	size: NonZeroUsize,
	align: Alignment,
}

unsafe impl TryToLayout for NonZst {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		Layout::from_size_align(self.size.get(), self.align.get()).map_err(Into::into)
	}
}

#[derive(Debug)]
pub struct Zst {
	align: Alignment,
}

unsafe impl ToLayout for Zst {
	fn to_layout(&self) -> Layout {
		Layout::from_size_align(0, self.align.get()).expect("unreachable")
	}
}
