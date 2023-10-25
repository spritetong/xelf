use std::{io, ptr::NonNull, slice};

trait RawPtrOps: Sized {
    #[inline(always)]
    fn as_ptr(&self) -> *const Self {
        self as *const _
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut _
    }

    #[inline(always)]
    unsafe fn iadd(&self, count: isize) -> &'static Self {
        &*(self as *const Self).wrapping_offset(count)
    }

    #[inline(always)]
    unsafe fn iadd_mut(&mut self, count: isize) -> &'static mut Self {
        &mut *(self as *mut Self).wrapping_offset(count)
    }

    #[inline(always)]
    unsafe fn uadd(&self, count: usize) -> &'static Self {
        &*(self as *const Self).wrapping_add(count)
    }

    #[inline(always)]
    unsafe fn uadd_mut(&mut self, count: usize) -> &'static mut Self {
        &mut *(self as *mut Self).wrapping_add(count)
    }

    #[inline(always)]
    fn idiff(&self, rhs: *const Self) -> isize {
        ((self as *const _ as isize) - (rhs as isize)) / std::mem::size_of::<Self>() as isize
    }

    #[inline(always)]
    fn udiff(&self, rhs: *const Self) -> usize {
        ((self as *const _ as usize) - (rhs as usize)) / std::mem::size_of::<Self>()
    }

    #[inline(always)]
    unsafe fn slice(&self, len: usize) -> &'static [Self] {
        slice::from_raw_parts(self, len)
    }

    #[inline(always)]
    unsafe fn slice_mut(&mut self, len: usize) -> &'static mut [Self] {
        slice::from_raw_parts_mut(self, len)
    }
}

trait SlicePtrOps {
    type Item: Sized;

    unsafe fn begin(&self) -> &'static Self::Item;
    unsafe fn begin_mut(&mut self) -> &'static mut Self::Item;

    unsafe fn end(&self) -> &'static Self::Item;
    unsafe fn end_mut(&mut self) -> &'static mut Self::Item;

    unsafe fn slice_unchecked_at(&self, at: usize) -> &[Self::Item];
    unsafe fn slice_unchecked_at_mut(&mut self, at: usize) -> &mut [Self::Item];

    fn slice_at(&self, at: usize) -> &[Self::Item];
    fn slice_at_mut(&mut self, at: usize) -> &mut [Self::Item];
}

impl<T: 'static + Sized> RawPtrOps for T {}

impl<T: 'static + Sized> SlicePtrOps for [T] {
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
