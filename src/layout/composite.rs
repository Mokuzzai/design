use super::*;

#[derive(Debug)]
pub struct Slice<L> {
	pub layout: L,
	pub length: usize,
}

impl<L> Slice<L> {
	pub fn new(layout: L, length: usize) -> Self {
		Self {
			length,
			layout,
		}
	}
	pub fn map<R>(self, f: impl FnOnce(L) -> R) -> Slice<R> {
		Slice::new(f(self.layout), self.length)
	}
}

unsafe impl<L: TryToLayout<Error = E>, E: Into<LayoutError>> TryToLayout for Slice<L> {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		let layout = self.layout.try_to_layout().map_err(Into::into)?;

		layout.repeat(self.length).map(|(layout, _)| layout).map_err(Into::into)
	}
}

pub struct Array<L, const LENGTH: usize> {
	pub layout: L,
}

impl<L, const LENGTH: usize> Array<L, LENGTH> {
	pub fn new(layout: L) -> Self {
		Self { layout }
	}
}

unsafe impl<L: TryToLayout<Error = E>, E: Into<LayoutError>, const LENGTH: usize> TryToLayout for Array<L, LENGTH> {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		let layout = self.layout.try_to_layout().map_err(Into::into)?;

		layout.repeat(LENGTH).map(|(layout, _)| layout).map_err(Into::into)
	}
}

#[derive(Debug)]
pub enum Multi {
	Layout(Layout),
	Slice(Box<Slice<Self>>),
	Struct(Struct),
}

unsafe impl TryToLayout for Multi {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		match *self {
			Self::Layout(ref l) => Ok(l.to_layout()),
			Self::Slice(ref l) => (&**l).try_to_layout(),
			Self::Struct(ref l) => l.try_to_layout(),
		}
	}
}

impl From<Layout> for Multi {
	fn from(l: Layout) -> Self {
		Self::Layout(l)
	}
}

impl<L: Into<Multi>> From<Slice<L>> for Multi {
	fn from(l: Slice<L>) -> Self {
		Self::Slice(Box::new(l.map(Into::into)))
	}
}

impl From<Struct> for Multi {
	fn from(l: Struct) -> Self {
		Self::Struct(l)
	}
}


#[derive(Debug)]
pub struct Field {
	pub offset: usize,
	pub layout: Multi,
}


#[derive(Debug)]
pub struct Struct {
	pub fields: Vec<Field>,
	pub unpadded_layout: Layout,
}

unsafe impl TryToLayout for Struct {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		Ok(self.unpadded_layout.pad_to_align())
	}
}

impl Struct {
	pub fn new() -> Self {
		Self {
			unpadded_layout: Layout::new::<()>(),
			fields: Vec::new(),
		}
	}
	pub fn push(&mut self, layout: impl Into<Multi>) -> Result<(), LayoutError> {
		let field_layout_builder = layout.into();
		let field_layout = field_layout_builder.try_to_layout()?;

		let (new_struct_layout, offset) = self.unpadded_layout.extend(field_layout)?;

		self.fields.push(Field { layout: field_layout_builder, offset });
		self.unpadded_layout = new_struct_layout;

		Ok(())
	}
}

#[test]
fn slb() {
	let mut slb = Struct::new();

	slb.push(Layout::new::<u32>()).unwrap();
	slb.push(Slice::new(Layout::new::<u8>(), 3)).unwrap();

	println!("{:?}", slb);
	println!("{:?}", slb.try_to_layout());

	#[repr(C)]
	struct Slb {
		_0: u32,
		_1: [u8; 3],
	}

	assert_eq!(slb.try_to_layout().unwrap(), Layout::new::<Slb>());
}
