use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::{mem, ptr};
use webrender::api::units::{DevicePixel, LayoutPixel};
use webrender::euclid::Length;

pub type LayoutLength = Length<f32, LayoutPixel>;
pub type DeviceLength = Length<f32, DevicePixel>;

/// Hashable floating-point storage for glyph size.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GlyphSize(pub f32);

impl Ord for GlyphSize {
    fn cmp(&self, other: &GlyphSize) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Eq for GlyphSize {}

impl Hash for GlyphSize {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl From<LayoutLength> for GlyphSize {
    fn from(size: LayoutLength) -> Self {
        GlyphSize(size.get())
    }
}

impl From<GlyphSize> for LayoutLength {
    fn from(size: GlyphSize) -> Self {
        Self::new(size.0)
    }
}

impl GlyphSize {
    pub fn zero() -> Self {
        GlyphSize(0.0)
    }

    pub fn from_layout_length(size: LayoutLength) -> Self {
        GlyphSize(size.get())
    }

    pub fn to_layout_length(&self) -> LayoutLength {
        LayoutLength::new(self.0)
    }
}

#[repr(C)]
pub struct WrVecU32 {
    /// `data` must always be valid for passing to Vec::from_raw_parts.
    /// In particular, it must be non-null even if capacity is zero.
    data: *mut u32,
    length: usize,
    capacity: usize,
}

impl WrVecU32 {
    pub fn into_vec(mut self) -> Vec<u32> {
        // Clear self and then drop self.
        self.flush_into_vec()
    }

    // Clears self without consuming self.
    pub fn flush_into_vec(&mut self) -> Vec<u32> {
        // Create a Vec using Vec::from_raw_parts.
        //
        // Here are the safety requirements, verbatim from the documentation of `from_raw_parts`:
        //
        // > * `ptr` must have been allocated using the global allocator, such as via
        // >   the [`alloc::alloc`] function.
        // > * `T` needs to have the same alignment as what `ptr` was allocated with.
        // >   (`T` having a less strict alignment is not sufficient, the alignment really
        // >   needs to be equal to satisfy the [`dealloc`] requirement that memory must be
        // >   allocated and deallocated with the same layout.)
        // > * The size of `T` times the `capacity` (ie. the allocated size in bytes) needs
        // >   to be the same size as the pointer was allocated with. (Because similar to
        // >   alignment, [`dealloc`] must be called with the same layout `size`.)
        // > * `length` needs to be less than or equal to `capacity`.
        // > * The first `length` values must be properly initialized values of type `T`.
        // > * `capacity` needs to be the capacity that the pointer was allocated with.
        // > * The allocated size in bytes must be no larger than `isize::MAX`.
        // >   See the safety documentation of [`pointer::offset`].
        //
        // These comments don't say what to do for zero-capacity vecs which don't have
        // an allocation. In particular, the requirement "`ptr` must have been allocated"
        // is not met for such vecs.
        //
        // However, the safety requirements of `slice::from_raw_parts` are more explicit
        // about the empty case:
        //
        // > * `data` must be non-null and aligned even for zero-length slices. One
        // >   reason for this is that enum layout optimizations may rely on references
        // >   (including slices of any length) being aligned and non-null to distinguish
        // >   them from other data. You can obtain a pointer that is usable as `data`
        // >   for zero-length slices using [`NonNull::dangling()`].
        //
        // For the empty case we follow this requirement rather than the more stringent
        // requirement from the `Vec::from_raw_parts` docs.
        let vec = unsafe { Vec::from_raw_parts(self.data, self.length, self.capacity) };
        self.data = ptr::NonNull::dangling().as_ptr();
        self.length = 0;
        self.capacity = 0;
        vec
    }

    pub fn as_slice(&self) -> &[u32] {
        unsafe { core::slice::from_raw_parts(self.data, self.length) }
    }
}

#[repr(C)]
pub struct WrVecU8 {
    /// `data` must always be valid for passing to Vec::from_raw_parts.
    /// In particular, it must be non-null even if capacity is zero.
    data: *mut u8,
    length: usize,
    capacity: usize,
}

impl WrVecU8 {
    pub fn into_vec(mut self) -> Vec<u8> {
        // Clear self and then drop self.
        self.flush_into_vec()
    }

    // Clears self without consuming self.
    pub fn flush_into_vec(&mut self) -> Vec<u8> {
        // Create a Vec using Vec::from_raw_parts.
        //
        // Here are the safety requirements, verbatim from the documentation of `from_raw_parts`:
        //
        // > * `ptr` must have been allocated using the global allocator, such as via
        // >   the [`alloc::alloc`] function.
        // > * `T` needs to have the same alignment as what `ptr` was allocated with.
        // >   (`T` having a less strict alignment is not sufficient, the alignment really
        // >   needs to be equal to satisfy the [`dealloc`] requirement that memory must be
        // >   allocated and deallocated with the same layout.)
        // > * The size of `T` times the `capacity` (ie. the allocated size in bytes) needs
        // >   to be the same size as the pointer was allocated with. (Because similar to
        // >   alignment, [`dealloc`] must be called with the same layout `size`.)
        // > * `length` needs to be less than or equal to `capacity`.
        // > * The first `length` values must be properly initialized values of type `T`.
        // > * `capacity` needs to be the capacity that the pointer was allocated with.
        // > * The allocated size in bytes must be no larger than `isize::MAX`.
        // >   See the safety documentation of [`pointer::offset`].
        //
        // These comments don't say what to do for zero-capacity vecs which don't have
        // an allocation. In particular, the requirement "`ptr` must have been allocated"
        // is not met for such vecs.
        //
        // However, the safety requirements of `slice::from_raw_parts` are more explicit
        // about the empty case:
        //
        // > * `data` must be non-null and aligned even for zero-length slices. One
        // >   reason for this is that enum layout optimizations may rely on references
        // >   (including slices of any length) being aligned and non-null to distinguish
        // >   them from other data. You can obtain a pointer that is usable as `data`
        // >   for zero-length slices using [`NonNull::dangling()`].
        //
        // For the empty case we follow this requirement rather than the more stringent
        // requirement from the `Vec::from_raw_parts` docs.
        let vec = unsafe { Vec::from_raw_parts(self.data, self.length, self.capacity) };
        self.data = ptr::NonNull::dangling().as_ptr();
        self.length = 0;
        self.capacity = 0;
        vec
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data, self.length) }
    }

    pub fn from_vec(mut v: Vec<u8>) -> WrVecU8 {
        let w = WrVecU8 {
            data: v.as_mut_ptr(),
            length: v.len(),
            capacity: v.capacity(),
        };
        mem::forget(v);
        w
    }

    pub fn reserve(&mut self, len: usize) {
        let mut vec = self.flush_into_vec();
        vec.reserve(len);
        *self = Self::from_vec(vec);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        let mut vec = self.flush_into_vec();
        vec.extend_from_slice(bytes);
        *self = Self::from_vec(vec);
    }
}

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum WrFontTemplate {
    Raw = 0,
    Native,
}

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct FontMetrics {
    pub min_width: ::libc::c_int,
    pub max_width: ::libc::c_int,
    pub pixel_size: ::libc::c_int,
    pub height: ::libc::c_int,
    pub space_width: ::libc::c_int,
    pub average_width: ::libc::c_int,
    pub ascent: ::libc::c_int,
    pub descent: ::libc::c_int,
    pub underline_thickness: ::libc::c_int,
    pub underline_position: ::libc::c_int,
    pub vertical_centering: bool,
    pub baseline_offset: ::libc::c_int,
}
