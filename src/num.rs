use std::str::FromStr;

pub trait Num: Sized + Copy + FromStr + Default {
    fn as_i64(self) -> i64;
    fn from_i64(value: i64) -> Self;

    fn as_u64(self) -> u64;
    fn from_u64(value: u64) -> Self;

    #[inline]
    fn as_usize(self) -> usize {
        self.as_i64() as usize
    }
}

pub trait Float: Sized + Copy + FromStr + Default {
    fn as_i64(self) -> i64;
    fn from_i64(value: i64) -> Self;

    fn as_u64(self) -> u64;
    fn from_u64(value: u64) -> Self;

    fn as_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! impl_convert {
    ($f:ident, $value:ident, $from:ty, $to:ty) => {
        #[inline(always)]
        fn $f(value: $from) -> $to {
            value as $to
        }
    };

    ($f:ident, $self:ident, $to:ty) => {
        #[inline(always)]
        fn $f($self) -> $to {
            $self as $to
        }
    };
}

macro_rules! impl_num {
    ($ty:ty) => {
        impl Num for $ty {
            impl_convert!(from_i64, value, i64, Self);
            impl_convert!(as_i64, self, i64);
            impl_convert!(from_u64, value, u64, Self);
            impl_convert!(as_u64, self, u64);
        }
    };
}

macro_rules! impl_float {
    ($ty:ty) => {
        impl Float for $ty {
            impl_convert!(from_i64, value, i64, Self);
            impl_convert!(as_i64, self, i64);
            impl_convert!(from_u64, value, u64, Self);
            impl_convert!(as_u64, self, u64);
            impl_convert!(from_f64, value, f64, Self);
            impl_convert!(as_f64, self, f64);
        }
    };
}

impl_num!(i8);
impl_num!(u8);
impl_num!(i16);
impl_num!(u16);
impl_num!(i32);
impl_num!(u32);
impl_num!(i64);
impl_num!(u64);
impl_num!(isize);
impl_num!(usize);

impl_float!(f32);
impl_float!(f64);
