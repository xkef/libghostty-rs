//! Idiomatic, safe Rust bindings for `libghostty-vt`, a terminal emulation library.
//!
//! # Memory management and lifetimes
//!
//! When creating the terminal and various other objects, you can control their
//! memory management via a **custom allocator**, usually specified with
//! methods like [`Terminal::new_with_alloc`]. Objects that accept allocators
//! are also bound by the `'alloc` lifetime, since they internally contain
//! a reference to the allocator. If you do not use a custom allocator,
//! feel free to always set the lifetime to `'static`.
//!
//! ## Using the unstable `Allocator` API
//!
//! You can adapt the existing, unstable `Allocator` API into a
//! [libghostty-friendly allocator](alloc::Allocator) via its `From`
//! implementation. Note that the `'alloc` lifetime must at least
//! live as long as the `Allocator` instance itself.
//!
//! # Thread safety
//!
//! All `libghostty-vt` objects are **not** thread-safe, and have been marked
//! `!Send + !Sync` accordingly. The expectation is for them to be managed
//! by a single thread, that may communicate with other threads via channels.
pub use ghostty_sys as ffi;

use std::marker::PhantomData;
use std::ptr::NonNull;

pub mod alloc;
pub mod build_info;
pub mod error;
pub mod fmt;
pub mod osc;
pub mod paste;
pub mod render;
pub mod sgr;
pub mod style;
pub mod terminal;

#[doc(inline)]
pub use crate::{
    error::Error,
    render::RenderState,
    terminal::{Options as TerminalOptions, Terminal},
};

use crate::error::{from_result, from_result_with_len};

pub const EXPORTED_API_SYMBOLS: &[&str] = ffi::EXPORTED_API_SYMBOLS;

// ---------------------------------------------------------------------------
// Focus encode
// ---------------------------------------------------------------------------

pub fn focus_encode(event: ffi::GhosttyFocusEvent, buf: &mut [u8]) -> Result<usize, Error> {
    let mut written: usize = 0;
    let result = unsafe {
        ffi::ghostty_focus_encode(event, buf.as_mut_ptr().cast(), buf.len(), &mut written)
    };
    from_result_with_len(result, written)
}

// ---------------------------------------------------------------------------
// RenderState
// ---------------------------------------------------------------------------

pub struct RenderState {
    ptr: NonNull<ffi::GhosttyRenderState>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl RenderState {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyRenderState_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_render_state_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn as_raw(&self) -> ffi::GhosttyRenderState_ptr {
        self.ptr.as_ptr()
    }

    pub fn update(&mut self, terminal: &mut Terminal) -> Result<(), Error> {
        let result =
            unsafe { ffi::ghostty_render_state_update(self.ptr.as_ptr(), terminal.as_raw()) };
        from_result(result)
    }

    pub fn dirty(&self) -> Result<ffi::GhosttyRenderStateDirty, Error> {
        let mut value: ffi::GhosttyRenderStateDirty =
            ffi::GhosttyRenderStateDirty_GHOSTTY_RENDER_STATE_DIRTY_FALSE;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_DIRTY,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn cols(&self) -> Result<u16, Error> {
        let mut value: u16 = 0;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_COLS,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn rows(&self) -> Result<u16, Error> {
        let mut value: u16 = 0;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_ROWS,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn populate_row_iterator(&self, iter: &mut RenderStateRowIterator) -> Result<(), Error> {
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_ROW_ITERATOR,
                std::ptr::from_mut(&mut iter.ptr).cast::<std::ffi::c_void>(),
            )
        };
        from_result(result)
    }

    pub fn cursor_visible(&self) -> Result<bool, Error> {
        let mut value = false;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_CURSOR_VISIBLE,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn cursor_viewport_has_value(&self) -> Result<bool, Error> {
        let mut value = false;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_HAS_VALUE,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn cursor_viewport_x(&self) -> Result<u16, Error> {
        let mut value: u16 = 0;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_X,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn cursor_viewport_y(&self) -> Result<u16, Error> {
        let mut value: u16 = 0;
        let result = unsafe {
            ffi::ghostty_render_state_get(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateData_GHOSTTY_RENDER_STATE_DATA_CURSOR_VIEWPORT_Y,
                std::ptr::from_mut(&mut value).cast(),
            )
        };
        from_result(result)?;
        Ok(value)
    }

    pub fn colors_get(&self) -> Result<ffi::GhosttyRenderStateColors, Error> {
        let mut colors = ffi::GhosttyRenderStateColors::default();
        colors.size = std::mem::size_of::<ffi::GhosttyRenderStateColors>();
        let result =
            unsafe { ffi::ghostty_render_state_colors_get(self.ptr.as_ptr(), &mut colors) };
        from_result(result)?;
        Ok(colors)
    }

    pub fn set_dirty(&mut self, dirty: ffi::GhosttyRenderStateDirty) -> Result<(), Error> {
        let result = unsafe {
            ffi::ghostty_render_state_set(
                self.ptr.as_ptr(),
                ffi::GhosttyRenderStateOption_GHOSTTY_RENDER_STATE_OPTION_DIRTY,
                std::ptr::from_ref(&dirty).cast(),
            )
        };
        from_result(result)
    }
}

impl Drop for RenderState {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_render_state_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// RenderStateRowIterator
// ---------------------------------------------------------------------------

fn render_state_row_iterator_next(ptr: NonNull<ffi::GhosttyRenderStateRowIterator>) -> bool {
    unsafe { ffi::ghostty_render_state_row_iterator_next(ptr.as_ptr()) }
}

fn render_state_row_get_dirty(
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
) -> Result<bool, Error> {
    let mut value = false;
    let result = unsafe {
        ffi::ghostty_render_state_row_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowData_GHOSTTY_RENDER_STATE_ROW_DATA_DIRTY,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

fn render_state_row_get_raw(
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
) -> Result<ffi::GhosttyRow, Error> {
    let mut value: ffi::GhosttyRow = 0;
    let result = unsafe {
        ffi::ghostty_render_state_row_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowData_GHOSTTY_RENDER_STATE_ROW_DATA_RAW,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

fn render_state_row_populate_cells(
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
    cells: &mut RenderStateRowCells,
) -> Result<(), Error> {
    let result = unsafe {
        ffi::ghostty_render_state_row_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowData_GHOSTTY_RENDER_STATE_ROW_DATA_CELLS,
            std::ptr::from_mut(&mut cells.ptr).cast::<std::ffi::c_void>(),
        )
    };
    from_result(result)
}

fn render_state_row_set_dirty(
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
    dirty: bool,
) -> Result<(), Error> {
    let result = unsafe {
        ffi::ghostty_render_state_row_set(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowOption_GHOSTTY_RENDER_STATE_ROW_OPTION_DIRTY,
            std::ptr::from_ref(&dirty).cast(),
        )
    };
    from_result(result)
}

fn render_state_row_cells_next(ptr: NonNull<ffi::GhosttyRenderStateRowCells>) -> bool {
    unsafe { ffi::ghostty_render_state_row_cells_next(ptr.as_ptr()) }
}

fn render_state_row_cell_get_raw(
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
) -> Result<ffi::GhosttyCell, Error> {
    let mut value: ffi::GhosttyCell = 0;
    let result = unsafe {
        ffi::ghostty_render_state_row_cells_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowCellsData_GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_RAW,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

fn render_state_row_cell_get_style(
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
) -> Result<ffi::GhosttyStyle, Error> {
    let mut value = ffi::GhosttyStyle::default();
    value.size = std::mem::size_of::<ffi::GhosttyStyle>();
    let result = unsafe {
        ffi::ghostty_render_state_row_cells_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowCellsData_GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_STYLE,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

fn render_state_row_cell_get_graphemes_len(
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
) -> Result<u32, Error> {
    let mut value: u32 = 0;
    let result = unsafe {
        ffi::ghostty_render_state_row_cells_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowCellsData_GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_GRAPHEMES_LEN,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

fn render_state_row_cell_get_graphemes_buf(
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
    buf: &mut [char],
) -> Result<(), Error> {
    let result = unsafe {
        ffi::ghostty_render_state_row_cells_get(
            ptr.as_ptr(),
            ffi::GhosttyRenderStateRowCellsData_GHOSTTY_RENDER_STATE_ROW_CELLS_DATA_GRAPHEMES_BUF,
            buf.as_mut_ptr().cast(),
        )
    };
    from_result(result)
}

pub struct RenderStateRowIterator {
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl RenderStateRowIterator {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyRenderStateRowIterator_ptr = std::ptr::null_mut();
        let result =
            unsafe { ffi::ghostty_render_state_row_iterator_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn advance(&mut self) -> bool {
        render_state_row_iterator_next(self.ptr)
    }

    pub fn dirty(&self) -> Result<bool, Error> {
        render_state_row_get_dirty(self.ptr)
    }

    pub fn raw_row(&self) -> Result<ffi::GhosttyRow, Error> {
        render_state_row_get_raw(self.ptr)
    }

    pub fn populate_cells(&self, cells: &mut RenderStateRowCells) -> Result<(), Error> {
        render_state_row_populate_cells(self.ptr, cells)
    }

    pub fn set_dirty(&mut self, dirty: bool) -> Result<(), Error> {
        render_state_row_set_dirty(self.ptr, dirty)
    }

    pub fn rows(&mut self) -> RenderStateRows<'_> {
        RenderStateRows {
            ptr: self.ptr,
            _not_send_sync: PhantomData,
        }
    }
}

pub struct RenderStateRows<'a> {
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
    _not_send_sync: PhantomData<&'a mut RenderStateRowIterator>,
}

/// View into the row currently selected by a `RenderStateRows` iterator.
///
/// This is a cursor view over the underlying C iterator state, not a copied
/// row snapshot. Advancing the parent iterator changes which row this view
/// points at.
pub struct RenderStateRow<'a> {
    ptr: NonNull<ffi::GhosttyRenderStateRowIterator>,
    _not_send_sync: PhantomData<&'a mut RenderStateRowIterator>,
}

impl<'a> Iterator for RenderStateRows<'a> {
    type Item = RenderStateRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if render_state_row_iterator_next(self.ptr) {
            Some(RenderStateRow {
                ptr: self.ptr,
                _not_send_sync: PhantomData,
            })
        } else {
            None
        }
    }
}

impl std::iter::FusedIterator for RenderStateRows<'_> {}

impl RenderStateRow<'_> {
    pub fn dirty(&self) -> Result<bool, Error> {
        render_state_row_get_dirty(self.ptr)
    }

    pub fn raw_row(&self) -> Result<ffi::GhosttyRow, Error> {
        render_state_row_get_raw(self.ptr)
    }

    pub fn populate_cells(&self, cells: &mut RenderStateRowCells) -> Result<(), Error> {
        render_state_row_populate_cells(self.ptr, cells)
    }

    pub fn set_dirty(&self, dirty: bool) -> Result<(), Error> {
        render_state_row_set_dirty(self.ptr, dirty)
    }
}

impl Drop for RenderStateRowIterator {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_render_state_row_iterator_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// RenderStateRowCells
// ---------------------------------------------------------------------------

pub struct RenderStateRowCells {
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl RenderStateRowCells {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyRenderStateRowCells_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_render_state_row_cells_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn advance(&mut self) -> bool {
        render_state_row_cells_next(self.ptr)
    }

    pub fn select(&mut self, x: u16) -> Result<(), Error> {
        let result = unsafe { ffi::ghostty_render_state_row_cells_select(self.ptr.as_ptr(), x) };
        from_result(result)
    }

    pub fn raw_cell(&self) -> Result<ffi::GhosttyCell, Error> {
        render_state_row_cell_get_raw(self.ptr)
    }

    pub fn style(&self) -> Result<ffi::GhosttyStyle, Error> {
        render_state_row_cell_get_style(self.ptr)
    }

    pub fn graphemes_len(&self) -> Result<u32, Error> {
        render_state_row_cell_get_graphemes_len(self.ptr)
    }

    pub fn graphemes_buf(&self, buf: &mut [char]) -> Result<(), Error> {
        render_state_row_cell_get_graphemes_buf(self.ptr, buf)
    }

    pub fn cells(&mut self) -> RenderStateCells<'_> {
        RenderStateCells {
            ptr: self.ptr,
            _not_send_sync: PhantomData,
        }
    }
}

pub struct RenderStateCells<'a> {
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
    _not_send_sync: PhantomData<&'a mut RenderStateRowCells>,
}

/// View into the cell currently selected by a `RenderStateCells` iterator.
///
/// This is a cursor view over the underlying C iterator state, not a copied
/// cell snapshot. Advancing the parent iterator changes which cell this view
/// points at.
pub struct RenderStateCell<'a> {
    ptr: NonNull<ffi::GhosttyRenderStateRowCells>,
    _not_send_sync: PhantomData<&'a mut RenderStateRowCells>,
}

impl<'a> Iterator for RenderStateCells<'a> {
    type Item = RenderStateCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if render_state_row_cells_next(self.ptr) {
            Some(RenderStateCell {
                ptr: self.ptr,
                _not_send_sync: PhantomData,
            })
        } else {
            None
        }
    }
}

impl std::iter::FusedIterator for RenderStateCells<'_> {}

impl RenderStateCell<'_> {
    pub fn raw_cell(&self) -> Result<ffi::GhosttyCell, Error> {
        render_state_row_cell_get_raw(self.ptr)
    }

    pub fn style(&self) -> Result<ffi::GhosttyStyle, Error> {
        render_state_row_cell_get_style(self.ptr)
    }

    pub fn graphemes_len(&self) -> Result<u32, Error> {
        render_state_row_cell_get_graphemes_len(self.ptr)
    }

    pub fn graphemes_buf(&self, buf: &mut [char]) -> Result<(), Error> {
        render_state_row_cell_get_graphemes_buf(self.ptr, buf)
    }
}

impl Drop for RenderStateRowCells {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_render_state_row_cells_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// KeyEvent
// ---------------------------------------------------------------------------

pub struct KeyEvent {
    ptr: NonNull<ffi::GhosttyKeyEvent>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl KeyEvent {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyKeyEvent_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_key_event_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn as_raw(&self) -> ffi::GhosttyKeyEvent_ptr {
        self.ptr.as_ptr()
    }

    pub fn set_action(&mut self, action: ffi::GhosttyKeyAction) {
        unsafe { ffi::ghostty_key_event_set_action(self.ptr.as_ptr(), action) }
    }

    pub fn get_action(&self) -> ffi::GhosttyKeyAction {
        unsafe { ffi::ghostty_key_event_get_action(self.ptr.as_ptr()) }
    }

    pub fn set_key(&mut self, key: ffi::GhosttyKey) {
        unsafe { ffi::ghostty_key_event_set_key(self.ptr.as_ptr(), key) }
    }

    pub fn get_key(&self) -> ffi::GhosttyKey {
        unsafe { ffi::ghostty_key_event_get_key(self.ptr.as_ptr()) }
    }

    pub fn set_mods(&mut self, mods: ffi::GhosttyMods) {
        unsafe { ffi::ghostty_key_event_set_mods(self.ptr.as_ptr(), mods) }
    }

    pub fn get_mods(&self) -> ffi::GhosttyMods {
        unsafe { ffi::ghostty_key_event_get_mods(self.ptr.as_ptr()) }
    }

    pub fn set_consumed_mods(&mut self, mods: ffi::GhosttyMods) {
        unsafe { ffi::ghostty_key_event_set_consumed_mods(self.ptr.as_ptr(), mods) }
    }

    pub fn get_consumed_mods(&self) -> ffi::GhosttyMods {
        unsafe { ffi::ghostty_key_event_get_consumed_mods(self.ptr.as_ptr()) }
    }

    pub fn set_composing(&mut self, composing: bool) {
        unsafe { ffi::ghostty_key_event_set_composing(self.ptr.as_ptr(), composing) }
    }

    pub fn get_composing(&self) -> bool {
        unsafe { ffi::ghostty_key_event_get_composing(self.ptr.as_ptr()) }
    }

    pub fn set_utf8(&mut self, text: Option<&[u8]>) {
        match text {
            Some(bytes) => unsafe {
                ffi::ghostty_key_event_set_utf8(
                    self.ptr.as_ptr(),
                    bytes.as_ptr().cast(),
                    bytes.len(),
                )
            },
            None => unsafe {
                ffi::ghostty_key_event_set_utf8(self.ptr.as_ptr(), std::ptr::null(), 0)
            },
        }
    }

    pub fn set_unshifted_codepoint(&mut self, codepoint: u32) {
        unsafe { ffi::ghostty_key_event_set_unshifted_codepoint(self.ptr.as_ptr(), codepoint) }
    }

    pub fn get_unshifted_codepoint(&self) -> u32 {
        unsafe { ffi::ghostty_key_event_get_unshifted_codepoint(self.ptr.as_ptr()) }
    }
}

impl Drop for KeyEvent {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_key_event_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// KeyEncoder
// ---------------------------------------------------------------------------

pub struct KeyEncoder {
    ptr: NonNull<ffi::GhosttyKeyEncoder>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl KeyEncoder {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyKeyEncoder_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_key_encoder_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn setopt(&mut self, option: ffi::GhosttyKeyEncoderOption, value: *const std::ffi::c_void) {
        unsafe { ffi::ghostty_key_encoder_setopt(self.ptr.as_ptr(), option, value) }
    }

    pub fn setopt_from_terminal(&mut self, terminal: &Terminal) {
        unsafe {
            ffi::ghostty_key_encoder_setopt_from_terminal(self.ptr.as_ptr(), terminal.as_raw())
        }
    }

    pub fn encode(&mut self, event: &KeyEvent, buf: &mut [u8]) -> Result<usize, Error> {
        let mut written: usize = 0;
        let result = unsafe {
            ffi::ghostty_key_encoder_encode(
                self.ptr.as_ptr(),
                event.as_raw(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut written,
            )
        };
        from_result_with_len(result, written)
    }
}

impl Drop for KeyEncoder {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_key_encoder_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// MouseEvent
// ---------------------------------------------------------------------------

pub struct MouseEvent {
    ptr: NonNull<ffi::GhosttyMouseEvent>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl MouseEvent {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyMouseEvent_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_mouse_event_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn as_raw(&self) -> ffi::GhosttyMouseEvent_ptr {
        self.ptr.as_ptr()
    }

    pub fn set_action(&mut self, action: ffi::GhosttyMouseAction) {
        unsafe { ffi::ghostty_mouse_event_set_action(self.ptr.as_ptr(), action) }
    }

    pub fn get_action(&self) -> ffi::GhosttyMouseAction {
        unsafe { ffi::ghostty_mouse_event_get_action(self.ptr.as_ptr()) }
    }

    pub fn set_button(&mut self, button: ffi::GhosttyMouseButton) {
        unsafe { ffi::ghostty_mouse_event_set_button(self.ptr.as_ptr(), button) }
    }

    pub fn clear_button(&mut self) {
        unsafe { ffi::ghostty_mouse_event_clear_button(self.ptr.as_ptr()) }
    }

    pub fn get_button(&self) -> Option<ffi::GhosttyMouseButton> {
        let mut button: ffi::GhosttyMouseButton = 0;
        let has_button =
            unsafe { ffi::ghostty_mouse_event_get_button(self.ptr.as_ptr(), &mut button) };
        if has_button { Some(button) } else { None }
    }

    pub fn set_mods(&mut self, mods: ffi::GhosttyMods) {
        unsafe { ffi::ghostty_mouse_event_set_mods(self.ptr.as_ptr(), mods) }
    }

    pub fn get_mods(&self) -> ffi::GhosttyMods {
        unsafe { ffi::ghostty_mouse_event_get_mods(self.ptr.as_ptr()) }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        let pos = ffi::GhosttyMousePosition { x, y };
        unsafe { ffi::ghostty_mouse_event_set_position(self.ptr.as_ptr(), pos) }
    }

    pub fn get_position(&self) -> ffi::GhosttyMousePosition {
        unsafe { ffi::ghostty_mouse_event_get_position(self.ptr.as_ptr()) }
    }
}

impl Drop for MouseEvent {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_mouse_event_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// MouseEncoder
// ---------------------------------------------------------------------------

pub struct MouseEncoder {
    ptr: NonNull<ffi::GhosttyMouseEncoder>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl MouseEncoder {
    pub fn new() -> Result<Self, Error> {
        let mut raw: ffi::GhosttyMouseEncoder_ptr = std::ptr::null_mut();
        let result = unsafe { ffi::ghostty_mouse_encoder_new(std::ptr::null(), &mut raw) };
        from_result(result)?;
        let ptr = NonNull::new(raw).ok_or(Error::OutOfMemory)?;
        Ok(Self {
            ptr,
            _not_send_sync: PhantomData,
        })
    }

    pub fn setopt(
        &mut self,
        option: ffi::GhosttyMouseEncoderOption,
        value: *const std::ffi::c_void,
    ) {
        unsafe { ffi::ghostty_mouse_encoder_setopt(self.ptr.as_ptr(), option, value) }
    }

    pub fn setopt_from_terminal(&mut self, terminal: &Terminal) {
        unsafe {
            ffi::ghostty_mouse_encoder_setopt_from_terminal(self.ptr.as_ptr(), terminal.as_raw())
        }
    }

    pub fn reset(&mut self) {
        unsafe { ffi::ghostty_mouse_encoder_reset(self.ptr.as_ptr()) }
    }

    pub fn encode(&mut self, event: &MouseEvent, buf: &mut [u8]) -> Result<usize, Error> {
        let mut written: usize = 0;
        let result = unsafe {
            ffi::ghostty_mouse_encoder_encode(
                self.ptr.as_ptr(),
                event.as_raw(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                &mut written,
            )
        };
        from_result_with_len(result, written)
    }
}

impl Drop for MouseEncoder {
    fn drop(&mut self) {
        unsafe { ffi::ghostty_mouse_encoder_free(self.ptr.as_ptr()) }
    }
}

// ---------------------------------------------------------------------------
// Cell / Row helpers
// ---------------------------------------------------------------------------``

pub fn cell_get_content_tag(cell: ffi::GhosttyCell) -> Result<ffi::GhosttyCellContentTag, Error> {
    let mut value: ffi::GhosttyCellContentTag = 0;
    let result = unsafe {
        ffi::ghostty_cell_get(
            cell,
            ffi::GhosttyCellData_GHOSTTY_CELL_DATA_CONTENT_TAG,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

pub fn cell_get_codepoint(cell: ffi::GhosttyCell) -> Result<u32, Error> {
    let mut value: u32 = 0;
    let result = unsafe {
        ffi::ghostty_cell_get(
            cell,
            ffi::GhosttyCellData_GHOSTTY_CELL_DATA_CODEPOINT,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

pub fn cell_get_color_palette(
    cell: ffi::GhosttyCell,
) -> Result<ffi::GhosttyColorPaletteIndex, Error> {
    let mut value: ffi::GhosttyColorPaletteIndex = 0;
    let result = unsafe {
        ffi::ghostty_cell_get(
            cell,
            ffi::GhosttyCellData_GHOSTTY_CELL_DATA_COLOR_PALETTE,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}

pub fn cell_get_color_rgb(cell: ffi::GhosttyCell) -> Result<ffi::GhosttyColorRgb, Error> {
    let mut value = ffi::GhosttyColorRgb::default();
    let result = unsafe {
        ffi::ghostty_cell_get(
            cell,
            ffi::GhosttyCellData_GHOSTTY_CELL_DATA_COLOR_RGB,
            std::ptr::from_mut(&mut value).cast(),
        )
    };
    from_result(result)?;
    Ok(value)
}
