#![feature(alloc_layout_extra)]
#![feature(ptr_metadata)]
#![feature(unsize)]

pub mod layout;
pub mod raw_struct;

use layout::Layout;
use layout::Result;
use layout::ToLayout;

use std::any::Any;
use std::ptr::DynMetadata;
use std::ptr::Pointee;
use std::marker::Unsize;

fn dyn_metadata<U, T: ?Sized + Any>() -> DynMetadata<T>
where
	T: Pointee<Metadata = DynMetadata<T>>,
	U: Unsize<T>,
{
	std::ptr::metadata(0 as *const U as *const T)
}

struct Field<T: ?Sized + Any> {
	metadata: DynMetadata<T>,
	offset: usize,
}

unsafe impl<T: ?Sized + Any> ToLayout for Field<T> {
	fn to_layout(&self) -> Layout {
		self.metadata.layout()
	}
}

pub struct HollowStruct<T: ?Sized + Any> {
	unpadded_layout: Layout,
	fields: Vec<Field<T>>,
}

impl<T: ?Sized + Any> HollowStruct<T> {
	pub fn new() -> Self {
		Self {
			unpadded_layout: Layout::new::<()>(),
			fields: Vec::new(),
		}
	}
	pub fn push<U>(&mut self) -> Result<usize>
	where
		T: Pointee<Metadata = DynMetadata<T>>,
		U: Unsize<T>,
	{
		let index = self.fields.len();

		let (new_layout, offset) = self.unpadded_layout.extend(Layout::new::<U>())?;
		let metadata = dyn_metadata::<U, T>();

		self.fields.push(Field { metadata, offset });
		self.unpadded_layout = new_layout;

		Ok(index)
	}
	pub fn with<U>(mut self) -> Result<Self>
	where
		T: Pointee<Metadata = DynMetadata<T>>,
		U: Unsize<T>,
	{
		self.push::<U>()?;

		Ok(self)
	}
}

unsafe impl<T: ?Sized + Any> ToLayout for HollowStruct<T> {
	fn to_layout(&self) -> Layout {
		self.unpadded_layout.pad_to_align()
	}
}

#[test]
fn test_hollow_struct() -> Result<()> {
	use std::fmt::Debug;

	#[repr(C)]
	#[derive(Debug, Eq, PartialEq)]
	struct Hs {
		_0: u32,
		_1: [u8; 3],
	}

	trait Object: Debug + Any {}

	impl<T: Debug + Any> Object for T {}

	let hs = HollowStruct::<dyn Object>::new()
		.with::<u32>()?
		.with::<[u8; 3]>()?
	;

	let a = Hs {
		_0: 33,
		_1: [1, 2, 3],
	};

	let mut b = raw_struct::BoxedStruct::uninit(hs).unwrap();

	let hs = b.layout();

	let _0 = hs.fields[0].offset;
	let _1 = hs.fields[1].offset;

	drop(hs);

	unsafe { b.as_mut_ptr().add(_0).cast::<u32>().write(33) };
	unsafe { b.as_mut_ptr().add(_1).cast::<[u8; 3]>().write([1, 2, 3]) };

	assert_eq!(a, *unsafe { &*b.as_ptr().cast::<Hs>() });

	Ok(())
}
