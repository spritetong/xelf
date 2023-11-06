#![allow(clippy::missing_safety_doc)]

use std::{ptr::NonNull, slice};

pub trait RawPtrOps: Sized {
    #[must_use]
    #[inline(always)]
    fn raw_ptr(&self) -> *const Self {
        self as *const _
    }

    #[must_use]
    #[inline(always)]
    fn raw_mut(&mut self) -> *mut Self {
        self as *mut _
    }

    #[must_use]
    #[inline(always)]
    unsafe fn iadd(&self, count: isize) -> &'static Self {
        &*(self as *const Self).wrapping_offset(count)
    }

    #[must_use]
    #[inline(always)]
    unsafe fn iadd_mut(&mut self, count: isize) -> &'static mut Self {
        &mut *(self as *mut Self).wrapping_offset(count)
    }

    #[must_use]
    #[inline(always)]
    unsafe fn uadd(&self, count: usize) -> &'static Self {
        &*(self as *const Self).wrapping_add(count)
    }

    #[must_use]
    #[inline(always)]
    unsafe fn uadd_mut(&mut self, count: usize) -> &'static mut Self {
        &mut *(self as *mut Self).wrapping_add(count)
    }

    #[must_use]
    #[inline(always)]
    fn idiff(&self, rhs: *const Self) -> isize {
        ((self as *const _ as isize) - (rhs as isize)) / std::mem::size_of::<Self>() as isize
    }

    #[must_use]
    #[inline(always)]
    fn udiff(&self, rhs: *const Self) -> usize {
        ((self as *const _ as usize) - (rhs as usize)) / std::mem::size_of::<Self>()
    }

    #[must_use]
    #[inline(always)]
    unsafe fn slice(&self, len: usize) -> &'static [Self] {
        slice::from_raw_parts(self, len)
    }

    #[must_use]
    #[inline(always)]
    unsafe fn slice_mut(&mut self, len: usize) -> &'static mut [Self] {
        slice::from_raw_parts_mut(self, len)
    }
}

pub trait SlicePtrOps {
    type Item: Sized;

    #[must_use]
    unsafe fn begin(&self) -> &'static Self::Item;
    #[must_use]
    unsafe fn begin_mut(&mut self) -> &'static mut Self::Item;

    #[must_use]
    unsafe fn end(&self) -> &'static Self::Item;
    #[must_use]
    unsafe fn end_mut(&mut self) -> &'static mut Self::Item;

    #[must_use]
    unsafe fn slice_unchecked_at(&self, at: usize) -> &[Self::Item];
    #[must_use]
    unsafe fn slice_unchecked_at_mut(&mut self, at: usize) -> &mut [Self::Item];

    #[must_use]
    fn slice_at(&self, at: usize) -> &[Self::Item];
    #[must_use]
    fn slice_at_mut(&mut self, at: usize) -> &mut [Self::Item];
}

impl<T: 'static + Copy + Sized> RawPtrOps for T {}

impl<T: 'static + Copy + Sized> SlicePtrOps for [T] {
    type Item = T;

    #[inline(always)]
    unsafe fn begin(&self) -> &'static Self::Item {
        &*self.as_ptr()
    }

    #[inline(always)]
    unsafe fn end(&self) -> &'static Self::Item {
        &*self.as_ptr().wrapping_add(self.len())
    }

    #[inline(always)]
    unsafe fn begin_mut(&mut self) -> &'static mut Self::Item {
        &mut *self.as_mut_ptr()
    }

    #[inline(always)]
    unsafe fn end_mut(&mut self) -> &'static mut Self::Item {
        &mut *self.as_mut_ptr().wrapping_add(self.len())
    }

    #[inline]
    unsafe fn slice_unchecked_at(&self, at: usize) -> &[Self::Item] {
        self.begin().uadd(at).slice(self.len() - at)
    }

    #[inline]
    unsafe fn slice_unchecked_at_mut(&mut self, at: usize) -> &mut [Self::Item] {
        self.begin_mut().uadd_mut(at).slice_mut(self.len() - at)
    }

    #[inline]
    fn slice_at(&self, at: usize) -> &[Self::Item] {
        unsafe {
            if at >= self.len() {
                slice::from_raw_parts(NonNull::dangling().as_ptr(), 0)
            } else {
                self.begin().uadd(at).slice(self.len() - at)
            }
        }
    }

    #[inline]
    fn slice_at_mut(&mut self, at: usize) -> &mut [Self::Item] {
        unsafe {
            if at >= self.len() {
                slice::from_raw_parts_mut(NonNull::dangling().as_ptr(), 0)
            } else {
                self.begin_mut().uadd_mut(at).slice_mut(self.len() - at)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Foo {
        a: i32,
        b: *const i32,
        c: *mut i32,
    }

    #[test]
    fn test_raw_ptr() {
        let a = Foo {
            a: 1,
            b: std::ptr::null(),
            c: std::ptr::null_mut(),
        };
        assert_eq!(a.idiff(&a), 0);
        assert_eq!(a.a.idiff(&a.a), 0);
        assert_eq!(a.b.idiff(&a.b), 0);
        assert_eq!(a.c.idiff(&a.c), 0);
    }
}
