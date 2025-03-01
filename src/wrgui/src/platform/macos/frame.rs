use std::ptr::NonNull;

use objc2::rc::{Id, Retained};
use objc2_app_kit::NSColor;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use webrender_api::ColorF;

use crate::types::{face, frame, ns_frame_scale_factor};

use super::color::{ns_color_to_color_f, pixel_to_color};
use super::types::OutputData;

impl frame {
    pub fn output_data(&self) -> Option<&OutputData> {
        unsafe { self.output_data.ns.as_ref() }
    }

    pub fn output_data_mut(&mut self) -> Option<&mut OutputData> {
        unsafe { self.output_data.ns.as_mut() }
    }

    pub fn cursor_color(&self) -> ColorF {
        ns_color_to_color_f(self.output_data().unwrap().cursor_color)
    }

    pub fn fg_color(&self) -> ColorF {
        ns_color_to_color_f(self.output_data().unwrap().foreground_color)
    }

    pub fn bg_color(&self) -> ColorF {
        ns_color_to_color_f(self.output_data().unwrap().background_color)
    }

    pub fn scale_factor(&mut self) -> f64 {
        unsafe { ns_frame_scale_factor(self) }
    }
}

impl face {
    pub fn fg_color(&self) -> Option<ColorF> {
        if self.foreground != 0 {
            return Some(pixel_to_color(self.foreground));
        }
        None
    }

    pub fn bg_color(&self) -> Option<ColorF> {
        if self.background != 0 {
            return Some(pixel_to_color(self.background));
        }
        None
    }
}

impl HasDisplayHandle for frame {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = AppKitDisplayHandle::new();
        let raw = RawDisplayHandle::AppKit(raw);
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

impl HasWindowHandle for frame {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if let Some(output_data) = self.output_data() {
            let handle =
                AppKitWindowHandle::new(unsafe { NonNull::new_unchecked(output_data.view) });
            let raw = RawWindowHandle::AppKit(handle);
            return Ok(unsafe { WindowHandle::borrow_raw(raw) });
        }
        Err(HandleError::Unavailable)
    }
}
