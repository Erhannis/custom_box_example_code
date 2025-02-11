use core::{marker::PhantomData, ptr::{NonNull}};

use heapless::Vec;

trait Valued {
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
  pointer: NonNull<T>,
  _marker: PhantomData<T>,
}

// pub fn getBin<T, U: ?Sized>(mut t: T) -> Bin<U>
// {
//   let n: NonNull<T> = NonNull::new(&mut t as *mut _).unwrap();
//   let bin: Bin<T> = Bin {
//     pointer: n,
//     _marker: PhantomData {},
//   };
//   return bin;
// }

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
  let binf: Bin<dyn Valued> = Bin {
    pointer: nf,
    _marker: PhantomData {},
  };

  let nb: NonNull<dyn Valued> = NonNull::new(&mut bar as *mut _).unwrap();
  let binb: Bin<dyn Valued> = Bin {
    pointer: nb,
    _marker: PhantomData {},
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

  // let foo = FooS {
  //   foo: 16,
  // };
  // let mut list: Vec<Bin<dyn Valued>, 16> = Vec::new();
  // let bf: Bin<dyn Valued> = getBin::<FooS, dyn Valued>(foo);
  // list.push(bf);
}