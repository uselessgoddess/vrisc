use std::ops::RangeInclusive;

pub struct ImmBuilder<T>(pub fn(T) -> T);

impl<T> ImmBuilder<T> {
  pub fn build(&self, x: T) -> T {
    (self.0)(x)
  }
}

trait BitSlice: Sized {
  fn bslice(self, range: RangeInclusive<usize>) -> Self;
}

macro_rules! bit_slice {
  ($($ty:ty)*) => {$(
    impl BitSlice for $ty {
      fn bslice(self, range: RangeInclusive<usize>) -> Self {
        let (start, end) = range.into_inner();
        let len = end - start;
        let mask = (2 << len) - 1;
        (self & (mask << start)) >> start
      }
    }
  )*};
}

bit_slice! { u8 u16 u32 u64 }
