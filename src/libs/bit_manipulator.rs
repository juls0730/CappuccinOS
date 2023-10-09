use core::{
    marker::PhantomData,
    ops::{BitAnd, BitOr, BitOrAssign, BitXorAssign, Not, Shl, Shr},
};

pub struct BitManipulator<T> {
    value: T,
    _phantom: PhantomData<T>,
}

impl<T> BitManipulator<T>
where
    T: From<u8>
        + Into<u64>
        + Copy
        + Shl<usize>
        + Shr<usize>
        + BitAnd<<<T as Shl<usize>>::Output as Not>::Output, Output = T>
        + BitOr<<T as Shl<usize>>::Output, Output = T>
        + BitAnd<<<T as Shr<usize>>::Output as Not>::Output, Output = T>
        + BitOr<<T as Shr<usize>>::Output, Output = T>,
    <T as Shr<usize>>::Output: Not,
    <T as Shr<usize>>::Output: BitAnd<T>,
    <T as Shr<usize>>::Output: BitOr<T>,
    <<T as Shr<usize>>::Output as BitAnd<T>>::Output: PartialEq<T>,
    <T as Shl<usize>>::Output: Not,
    <T as Shl<usize>>::Output: BitAnd<T>,
    <T as Shl<usize>>::Output: BitOr<T>,
    <<T as Shl<usize>>::Output as BitAnd<T>>::Output: PartialEq<T>,
{
    #[inline]
    pub fn set_bit(&mut self, position: usize) {
        self.value = self.value | (T::from(1) << position);
    }

    #[inline]
    pub fn unset_bit(&mut self, position: usize) {
        self.value = self.value & !(T::from(1) << position);
    }

    #[inline]
    pub fn extract_bit(&self, position: usize) -> bool {
        return ((self.value >> position) & T::from(1)) != T::from(0);
    }

    #[inline]
    pub fn get(&self) -> T {
        return self.value;
    }

    #[inline]
    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
    }
}

impl BitManipulator<u8> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            value: 0,
            _phantom: PhantomData,
        }
    }

    pub const fn new_from(value: u8) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }
}

impl BitXorAssign<u8> for BitManipulator<u8> {
    #[inline]
    fn bitxor_assign(&mut self, other: u8) {
        self.value ^= other;
    }
}

impl BitOrAssign<u8> for BitManipulator<u8> {
    #[inline]
    fn bitor_assign(&mut self, other: u8) {
        self.value |= other;
    }
}
