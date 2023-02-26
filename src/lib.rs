#![feature(alloc_layout_extra)]

pub mod layout;

use layout::Layout;
use layout::TryToLayout;

use std::ptr::NonNull;

pub struct RawBoxedStruct {
	ptr: NonNull<u8>,
}

impl RawBoxedStruct {
	#[must_use = "Letting `RawBoxedStruct` drop leaks memory"]
	pub fn alloc(layout: Layout) -> Self {
		if layout.size() == 0 {
			Self {
				ptr: unsafe { NonNull::new_unchecked(layout.align() as *mut u8) },
			}
		} else {
			let Some(ptr) = NonNull::new(unsafe { std::alloc::alloc(layout) }) else {
				std::alloc::handle_alloc_error(layout)
			};

			Self {
				ptr,
			}
		}
	}
	pub unsafe fn dealloc(&mut self, layout: Layout) {
		if layout.size() == 0 {
			// no-op, we never allocated
		} else {
			std::alloc::dealloc(self.ptr.as_ptr(), layout)
		}
	}
}

pub struct BoxedStruct<L: TryToLayout> {
	inner: RawBoxedStruct,
	layout: L,
}

impl<L: TryToLayout> BoxedStruct<L> {
	pub fn uninit(layout: L) -> Result<Self, L::Error> {
		let l = layout.try_to_layout()?;

		Ok(Self {
			inner: RawBoxedStruct::alloc(l),
			layout,
		})
	}
}

impl<L: TryToLayout> Drop for BoxedStruct<L> {
	fn drop(&mut self) {
		unsafe { self.inner.dealloc(self.layout.try_to_layout().unwrap_unchecked()) }
	}
}
