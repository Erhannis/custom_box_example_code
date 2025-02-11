#![feature(unsize)] // *Ideally* I wouldn't need these
#![feature(coerce_unsized)]

use core::{marker::{PhantomData, Unsize}, ptr::NonNull, ops::CoerceUnsized};

use heapless::Vec;

pub trait Valued {
  fn get_value(&self) -> u32;
}

struct FooS {
  foo: u32,
}

struct BarS { // Didn't get to the point where I could test the difference
  bar: u32,
}

impl Valued for FooS {
    fn get_value(&self) -> u32 {
        return self.foo;
    }
}

impl Valued for BarS {
    fn get_value(&self) -> u32 {
        return self.bar;
    }
}

impl Drop for FooS { // These were so I could test the value wasn't dropped prematurely
  fn drop(&mut self) {
    println!("Drop FooS {}", self.foo);
  }
}

impl Drop for BarS {
  fn drop(&mut self) {
    println!("Drop BarS {}", self.bar);
  }
}


pub struct Bin<T: ?Sized> {
  data: [u8; 128], // Eventually this would be configurable
  pointer: NonNull<T>,
  _marker: PhantomData<T>,
}

impl<T: ?Sized> Bin<T> {
  pub fn get(&self) -> &T {
    unsafe {
      return self.pointer.as_ref();
    }
  }

  pub fn get_mut(&mut self) -> &mut T {
    unsafe {
      return self.pointer.as_mut();
    }
  }
}

/*
// This MIGHT be key to fixing the problem, except it seems the type parameter syntax can't express "where the input type implements the output type"
impl<T> Bin<T> {
  pub fn toUnsized<U: ?Sized>(self) -> U
  where T: U // "expected trait, found type parameter `U`"
  {
    ...
  }
}

// I thought maybe if I used the specific trait I could at least solve half the problem and test if the idea works.
// Did not work; still read memory wrong.
impl<T: 'static> Bin<T> {
  pub fn toUnsized(mut self) -> Bin<dyn Valued>
  where T: Valued {
    let mut a = self.get_mut();
    let mut b = a as &mut dyn Valued;
    let c: &mut dyn Valued = unsafe {
      core::ptr::read(&raw mut b)
    };
    let n: NonNull<dyn Valued> = NonNull::new(c as *mut _).unwrap();
    let r: Bin<dyn Valued> = Bin {
      data: self.data,
      pointer: n,
      _marker: PhantomData {},
    };
    return r;
  }
}
*/


// I suspect this is not working right.
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Bin<U>> for Bin<T> {}

pub fn get_bin<T>(mut t: T) -> Bin<T>
{
  let n: NonNull<T> = NonNull::new(&mut t as *mut _).unwrap();
  let mut bin: Bin<T> = Bin {
    data: [0; 128],
    pointer: n,
    _marker: PhantomData {},
  };
  let a = (&raw mut bin.data);
  let b = a as *mut T;

  unsafe {
    core::ptr::write(b, t); //LEAK I should drop the contents when the wrapper is dropped
  }
  bin.pointer = NonNull::new(b).unwrap();

  return bin;
}

pub fn main() {
  let foo = FooS {
    foo: 16,
  };
  let mut list: Vec<Bin<dyn Valued>, 16> = Vec::new();
  let bf = get_bin::<FooS>(foo);
  println!("FooS value: {}", bf.get().get_value()); // "FooS value: 16"
  let bf2: Bin<dyn Valued> = bf as Bin<dyn Valued>;
  println!("Valued value: {}", bf2.get().get_value()); // "Valued value: 1432019472"
  // let bf2: Bin<dyn Valued> = bf.toUnsized();
  list.push(bf2).unwrap_or_else(|_| panic!("Something went horribly wrong!"));
  println!("list[0] value: {}", list[0].get().get_value()); // "list[0] value: 4294954704"
  // Notably, list[0] is a different wrong value.  Maybe there's a pointer not updated in the move.
}