//! Runtime-swappable functions for operations that depend on
//! external implementations (e.g. image decoding).
//!
//! These are process-global settings that must be configured at startup
//! before any terminal functionality that depends on them is used.
//! Setting these enables various optional features of the terminal. For
//! example, setting a PNG decoder enables PNG image support in the Kitty
//! Graphics Protocol.
//!
//! Use the various `set_*` functions to install or clear an implementation.
//! Passing `None` as the value clears the implementation and disables the
//! corresponding feature.
use std::cell::Cell;

use crate::{
    alloc::{Allocator, Bytes},
    error::{Result, from_result},
    ffi,
};

thread_local! {
    static VTABLE: Cell<VTable> = Cell::new(VTable::default());
}
#[derive(Clone, Copy)]
struct VTable {
    decode_png: Option<&'static dyn DecodePngFn>,
}
impl Default for VTable {
    fn default() -> Self {
        Self { decode_png: None }
    }
}

/// Set the PNG decode function.
///
/// When set, the terminal can accept PNG images via the Kitty Graphics Protocol.
/// When cleared (`None` value), PNG decoding is unsupported and PNG image data
/// will be rejected.
#[cfg(feature = "kitty-images")]
pub fn set_decode_png(f: Option<&'static dyn DecodePngFn>) -> Result<()> {
    unsafe extern "C" fn callback(
        _userdata: *mut std::ffi::c_void,
        allocator: *const ffi::Allocator,
        data: *const u8,
        data_len: usize,
        out: *mut ffi::SysImage,
    ) -> bool {
        let Some(func) = VTABLE.get().decode_png else {
            return false;
        };
        // SAFETY: We trust libghostty to return valid values.
        let alloc = unsafe { Allocator::from_raw(allocator) };
        let data = unsafe { std::slice::from_raw_parts(data, data_len) };

        match func(&alloc, data) {
            Some(result) => {
                unsafe { *out = result.into() };
                true
            }
            None => false,
        }
    }

    VTABLE.with(|vt| VTable {
        decode_png: f,
        ..vt.get()
    });

    let ptr = match f {
        None => std::ptr::null(),
        Some(_) => {
            let ptr: unsafe extern "C" fn(
                userdata: *mut std::ffi::c_void,
                allocator: *const ffi::Allocator,
                data: *const u8,
                data_len: usize,
                out: *mut ffi::SysImage,
            ) -> bool = callback;
            ptr as *const std::ffi::c_void
        }
    };
    set(ffi::SysOption::GHOSTTY_SYS_OPT_DECODE_PNG, ptr)
}

fn set<T>(opt: ffi::SysOption::Type, val: *const T) -> Result<()> {
    let result = unsafe { ffi::ghostty_sys_set(opt, val.cast()) };
    from_result(result)
}

/// Callback type for [`set_decode_png`].
#[cfg(feature = "kitty-images")]
pub trait DecodePngFn:
    for<'alloc, 'ctx> Fn(&'alloc Allocator<'ctx>, &[u8]) -> Option<Image<'alloc>> + 'static
{
}
#[cfg(feature = "kitty-images")]
impl<F> DecodePngFn for F where
    F: for<'alloc, 'ctx> Fn(&'alloc Allocator<'ctx>, &[u8]) -> Option<Image<'alloc>> + 'static
{
}

/// Implementation for [`set_decode_png`] using the [`png`] crate.
///
/// ```rust
/// use ghostty::sys;
///
/// sys::set_decode_png(sys::decode_with_png_crate);
/// ```
#[cfg(all(feature = "kitty-images", feature = "png"))]
pub fn decode_with_png_crate<'alloc, 'ctx>(
    alloc: &'alloc Allocator<'ctx>,
    data: &[u8],
) -> Option<Image<'alloc>> {
    use png::{Decoder, Transformations};
    use std::io::Cursor;

    let mut decoder = Decoder::new(Cursor::new(data));

    // libghostty only accepts RGBA8 data, so we have to apply some
    // transformations to accept images in other formats, namely
    // expanding palette and grayscale colors to RGBA8 and stripping
    // 16-bit color depth information back down into 8-bit.
    decoder.set_transformations(Transformations::ALPHA | Transformations::STRIP_16);

    let mut frame = decoder.read_info().ok()?;
    let mut buf = vec![0u8; frame.output_buffer_size()?];
    let info = frame.next_frame(&mut buf).ok()?;

    let mut bytes = Bytes::new_with_alloc(alloc, info.buffer_size()).ok()?;
    bytes.copy_from_slice(&buf[..info.buffer_size()]);
    frame.finish().ok()?;

    Some(Image {
        width: info.width,
        height: info.height,
        data: bytes,
    })
}

/// Result of decoding an image.
///
/// The `data` buffer must be allocated through the allocator provided to the
/// decode callback. The library takes ownership and will free it with the
/// same allocator.
#[derive(Debug)]
pub struct Image<'alloc> {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Byte buffer containing the decoded RGBA pixel data.
    pub data: Bytes<'alloc>,
}
impl From<Image<'_>> for ffi::SysImage {
    fn from(mut value: Image<'_>) -> Self {
        Self {
            width: value.width,
            height: value.height,
            data: value.data.as_mut_ptr(),
            data_len: value.data.len(),
        }
    }
}
