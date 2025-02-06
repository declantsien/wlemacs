use crate::frame::FrameRef;
use raw_window_handle::DisplayHandle;
use raw_window_handle::HandleError;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use raw_window_handle::WindowHandle;

impl FrameRef {
    pub fn cursor_color(&self) -> ::libc::c_ulong {
        self.output().cursor_color
    }

    pub fn cursor_foreground_color(&self) -> ::libc::c_ulong {
        self.output().cursor_foreground_color
    }

    pub fn scale_factor(&self) -> f64 {
        //TODO
        1.0
    }
}

impl HasWindowHandle for FrameRef {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = WaylandWindowHandle::new({
            let output = self.output();
            let ptr = output.surface;
            std::ptr::NonNull::new(ptr as *mut _).expect("wl_surface will never be null")
        });

        unsafe { Ok(WindowHandle::borrow_raw(raw.into())) }
    }
}

impl HasDisplayHandle for FrameRef {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = WaylandDisplayHandle::new({
            let dpyinfo = self.display_info();
            let ptr = dpyinfo.display;
            std::ptr::NonNull::new(ptr as *mut _).expect("wl_display should never be null")
        });

        unsafe { Ok(DisplayHandle::borrow_raw(raw.into())) }
    }
}
