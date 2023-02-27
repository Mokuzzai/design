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
	Struct(Struct<Self>),
}

unsafe impl TryToLayout for Multi {
	type Error = LayoutError;

	fn try_to_layout(&self) -> Result<Layout, Self::Error> {
		match *self {
			Self::Layout(ref l) => Ok(l.to_layout()),
			Self::Struct(ref l) => Ok(l.to_layout()),
			Self::Slice(ref l) => (&**l).try_to_layout(),
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

impl From<Struct<Self>> for Multi {
	fn from(l: Struct<Self>) -> Self {
		Self::Struct(l)
	}
}


#[derive(Debug)]
pub struct Field<L> {
	pub layout: L,
	pub offset: usize,
}


#[derive(Debug)]
pub struct Struct<L> {
	pub fields: Vec<Field<L>>,
	pub unpadded_layout: Layout,
}

unsafe impl<L> ToLayout for Struct<L> {
	fn to_layout(&self) -> Layout {
		self.unpadded_layout.pad_to_align()
	}
}

impl<L> Struct<L> {
	pub fn new() -> Self {
		Self {
			unpadded_layout: Layout::new::<()>(),
			fields: Vec::new(),
		}
	}
	pub fn len(&self) -> usize {
		self.fields.len()
	}
	pub fn fields(&self) -> &[Field<L>] {
		&self.fields
	}
}

impl<L, E> Struct<L>
where
	L: TryToLayout<Error = E>,
	E: Into<LayoutError>,
{
	pub fn push(&mut self, field_layout_builder: L) -> Result<usize, LayoutError> {
		let index = self.len();

		let field_layout = field_layout_builder.try_to_layout().map_err(Into::into)?;

		let (new_struct_layout, offset) = self.unpadded_layout.extend(field_layout)?;

		self.fields.push(Field { layout: field_layout_builder, offset });
		self.unpadded_layout = new_struct_layout;

		Ok(index)
	}
	pub fn with(mut self, field_layout_builder: L) -> Result<Self, LayoutError> {
		self.push(field_layout_builder)?;
		Ok(self)
	}
}

#[test]
fn slb() {
	let mut slb = Struct::<Multi>::new();

	slb.push(Layout::new::<u32>().into()).unwrap();
	slb.push(Slice::new(Layout::new::<u8>(), 3).into()).unwrap();

	println!("{:?}", slb);
	println!("{:?}", slb.try_to_layout());

	#[repr(C)]
	struct Slb {
		_0: u32,
		_1: [u8; 3],
	}

	assert_eq!(slb.try_to_layout().unwrap(), Layout::new::<Slb>());
}
