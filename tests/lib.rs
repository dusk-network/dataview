use rand::Rng;
use std::marker::PhantomData;
use std::mem;

use dataview::Pod;
use quickcheck::{quickcheck, Arbitrary, Gen, StdGen};

const N: usize = 8;

#[derive(Default, Clone, Copy, Pod, Debug, PartialEq)]
struct TwoU32 {
	#[allow(unused)]
	a: u32,
	#[allow(unused)]
	b: u32,
}

impl Arbitrary for TwoU32 {
	fn arbitrary<G: Gen>(g: &mut G) -> Self {
		TwoU32 {
			a: g.next_u32(),
			b: g.next_u32(),
		}
	}
}

#[derive(Default, Clone, Copy, Pod, Debug, PartialEq)]
struct Pair {
	#[allow(unused)]
	a: TwoU32,
	#[allow(unused)]
	b: TwoU32,
}

impl Arbitrary for Pair {
	fn arbitrary<G: Gen>(g: &mut G) -> Self {
		Pair {
			a: TwoU32::arbitrary(g),
			b: TwoU32::arbitrary(g),
		}
	}
}

#[derive(Default, Clone, Copy, Pod, Debug, PartialEq)]
struct ZeroSize {
	#[allow(unused)]
	a: (),
	#[allow(unused)]
	b: PhantomData<Box<u32>>,
}

impl Arbitrary for ZeroSize {
	fn arbitrary<G: Gen>(_: &mut G) -> Self {
		ZeroSize {
			a: (),
			b: PhantomData,
		}
	}
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct NotPossible {
	a: u32,
	b: u16,
}

#[derive(Debug, Clone)]
pub enum Op {
	Mutate(usize),
	ConvertToBytes(usize),
	ConvertFromBytes(usize),
}

impl Arbitrary for Op {
	fn arbitrary<G: Gen>(g: &mut G) -> Self {
		let k = g.gen_range(0, N);
		let op = g.gen_range(0, 3);
		match op {
			0 => Op::Mutate(k),
			1 => Op::ConvertToBytes(k),
			2 => Op::ConvertFromBytes(k),
			_ => unreachable!(),
		}
	}
}

fn test_with_type<T>(ops: Vec<Op>)
where
	T: Sized + Default + Copy + Pod + Arbitrary + std::fmt::Debug + PartialEq,
{
	let t_size = mem::size_of::<T>();
	let byte_size = N * t_size;

	// Set up a vector of bytes of appropriate length
	let mut bytes = {
		let mut vec = Vec::with_capacity(byte_size);
		for _ in 0..byte_size {
			vec.push(0u8)
		}
		vec
	};

	// Initialize an array
	let mut array = [T::default(); N];

	for op in ops {
		match op {
			Op::Mutate(key) => {
				// Create a random T and write it as a value and as bytes

				let mut gen = StdGen::new(rand::thread_rng(), 256);
				let arb = T::arbitrary(&mut gen);

				bytes.as_data_view_mut().write(key * t_size, &arb);
				array[key] = arb;
			}

			Op::ConvertToBytes(key) => {
				// Copy from array to bytes
				bytes.as_data_view_mut().write(key * t_size, &array[key]);
			}

			Op::ConvertFromBytes(key) => {
				// Copy from bytes to array
				array[key] = *bytes.as_data_view().read(key * t_size);
			}
		}
		// Assert that the byte contents are the same
		assert_eq!(array.as_bytes(), &*bytes);
		// Assert that the slice contents are the same
		assert_eq!(&array, bytes.as_data_view().slice(0, N));
	}
}

quickcheck! {
	fn qc(ops: Vec<Op>) -> bool {
		  test_with_type::<u32>(ops.clone());
		  test_with_type::<TwoU32>(ops.clone());
		  test_with_type::<Pair>(ops.clone());
		  test_with_type::<()>(ops.clone());
		  test_with_type::<ZeroSize>(ops);
	  true
  }
}
