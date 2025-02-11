use core::{marker::PhantomData, ptr::NonNull};
use std::{marker::Unsize, ops::CoerceUnsized};

use heapless::Vec;

pub trait Valued {
  fn getValue(&self) -> u32;
}

struct FooS {
  foo: u32,
}

struct BarS {
  bar: u32,
}

impl Valued for FooS {
    fn getValue(&self) -> u32 {
        return self.foo;
    }
}

impl Valued for BarS {
    fn getValue(&self) -> u32 {
        return self.bar;
    }
}

impl Drop for FooS {
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
  data: [u8; 128], //DUMMY size
  pointer: NonNull<T>,
  _marker: PhantomData<T>,
}

impl<T: ?Sized> Bin<T> {
  pub fn get(&self) -> &T {
    unsafe {
      return self.pointer.as_ref();
    }
  }

  pub fn getMut(&mut self) -> &mut T {
    unsafe {
      return self.pointer.as_mut();
    }
  }
}

/* // This MIGHT be key to fixing the problem, except it seems the type parameter syntax can't express "where the input type implements the output type"
impl<T> Bin<T> {
  pub fn toUnsized<U: ?Sized>(self) -> U
  where T: U // "expected trait, found type parameter `U`"
  {
    ...
  }
}
*/

//DUMMY I don't think this is working right
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Bin<U>> for Bin<T> {}

pub fn getBin<T>(mut t: T) -> Bin<T>
{
  let n: NonNull<T> = NonNull::new(&mut t as *mut _).unwrap();
  let mut bin: Bin<T> = Bin {
    pointer: n,
    _marker: PhantomData {},
    data: [0; 128],
  };
  let a = (&raw mut bin.data);
  let b = a as *mut T;

  println!("gb 1");
  unsafe {
    println!("gb 2");
    core::ptr::write(b, t); //LEAK I should drop the contents when the wrapper is dropped
    println!("gb 2.1");
    // let mut t2 = leak(t);
    println!("gb 3");
    // *b = c; // Copy bytes into `data`, I think?
    println!("gb 4");
  }
  println!("gb 5");
  //DUMMY Leak t?  Trying to do this says t was moved, maybe we're ok?
  //        No, Foo's drop is called.  ...I think that's bad, right???
  // core::mem::forget(t);
  bin.pointer = NonNull::new(b).unwrap();
  println!("gb 6");

  return bin;
}

fn leak<'a, T>(t: T) -> &'a mut T {
  unsafe {
    let mut b = core::mem::ManuallyDrop::new(t);
    let ptr = &raw mut *b;
    let b = &raw mut *&mut *ptr;
    return &mut *b;
  };
}

pub fn test11() -> (Bin<dyn Valued>, Bin<dyn Valued>) {
  // let v: Vec<dyn Valued, 16> = Vec::new(); // Fails because size not known
  let mut foo = FooS {
    foo: 4,
  };
  let mut bar = BarS {
    bar: 5,
  };

  // let uf: Unique<dyn Valued> = Unique::new(&mut foo as *mut _).unwrap();
  let nf: NonNull<dyn Valued> = NonNull::new(&mut foo as *mut _).unwrap();
  let mut binf: Bin<dyn Valued> = Bin {
    pointer: nf,
    _marker: PhantomData {},
    data: [0; 128],
  };

  let nb: NonNull<dyn Valued> = NonNull::new(&mut bar as *mut _).unwrap();
  let binb: Bin<dyn Valued> = Bin {
    pointer: nb,
    _marker: PhantomData {},
    data: [0; 128],
  };

  //DUMMY I'm not sure the above respects simultaneous mutable reference law

  return (binf, binb);
}

pub fn test1() {
  let (binf, binb) = test11();
  //DUMMY //LEAK foo and bar are dropped here, the following is UB
  let vf = unsafe {
    binf.pointer.as_ref().getValue()
  };
  println!("vf {}", vf);
  let vb = unsafe {
    binb.pointer.as_ref().getValue()
  };
  println!("vb {}", vb);

  println!("create foo:");
  let foo = FooS {
    foo: 16,
  };
  println!("create list:");
  let mut list: Vec<Bin<dyn Valued>, 16> = Vec::new();
  println!("wrap foo:");
  let bf = getBin::<FooS>(foo);
  println!("cast bin:");
  let bf2: Bin<dyn Valued> = bf as Bin<dyn Valued>;
  println!("push bin into list:");
  list.push(bf2).unwrap_or_else(|_| panic!("Something went horribly wrong!"));
  println!("end of function");
  println!("foo value: {}", list[0].get().getValue());
}