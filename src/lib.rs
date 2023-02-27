#![feature(alloc_layout_extra)]
#![feature(ptr_metadata)]
#![feature(unsize)]

pub mod layout;
pub mod raw_struct;

use std::any::Any;
use std::ptr::DynMetadata;
use std::ptr::Pointee;
use std::marker::Unsize;

pub fn dyn_metadata<U, T: ?Sized + Any>() -> DynMetadata<T>
where
	T: Pointee<Metadata = DynMetadata<T>>,
	U: Unsize<T>,
{
	std::ptr::metadata(0 as *const U as *const T)
}

pub type HollowStruct<T> = layout::composite::Struct<DynMetadata<T>>;

#[test]
fn test_hollow_struct() -> layout::Result<()> {
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
		.with(dyn_metadata::<u32, _>())?
		.with(dyn_metadata::<[u8; 3], _>())?
	;

	let a = Hs {
		_0: 33,
		_1: [1, 2, 3],
	};

	let mut b = raw_struct::BoxedStruct::uninit(hs).unwrap();

	let hs = b.layout();

	let _0 = hs.fields()[0].offset;
	let _1 = hs.fields()[1].offset;

	drop(hs);

	unsafe { b.as_mut_ptr().add(_0).cast::<u32>().write(33) };
	unsafe { b.as_mut_ptr().add(_1).cast::<[u8; 3]>().write([1, 2, 3]) };

	assert_eq!(a, *unsafe { &*b.as_ptr().cast::<Hs>() });

	Ok(())
}
