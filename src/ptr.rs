use ::std::mem::transmute;

/// Convert a const raw pointer to a static reference.
///
/// This operation is **UNSAFE**.
#[inline(always)]
pub fn unsafe_ref<T>(p: *const T) -> &'static T {
    debug_assert!(!p.is_null());
    unsafe { transmute(p) }
}

/// Convert a const/mutable raw pointer to a static mutable reference.
///
/// This operation is **UNSAFE**.
#[inline(always)]
pub fn unsafe_mut<T>(p: *const T) -> &'static mut T {
    debug_assert!(!p.is_null());
    unsafe { transmute(p) }
}
