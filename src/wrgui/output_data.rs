use std::{cell::Cell, collections::HashMap};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    surface::{Surface, WindowSurface},
};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event::{ElementState, Ime, TouchPhase, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::ModifiersState,
    window::{CursorIcon, Window as WinitWindow, WindowAttributes, WindowId},
};

#[derive(Default)]
pub(crate) struct EventListeners {
    /// This is `true` if the controller wants to get and handle OnNavigationStarting/AllowNavigationRequest
    pub(crate) on_navigation_starting: bool,
    /// A id to request response sender map if the controller wants to get and handle web resource requests
    // pub(crate) on_web_resource_requested:
    //     Option<HashMap<uuid::Uuid, (url::Url, IpcSender<WebResourceResponseMsg>)>>,
    /// This is `true` if the controller wants to get and handle WindowEvent::CloseRequested
    pub(crate) on_close_requested: bool,
}

/// A Verso window is a Winit window containing several web views.
pub struct OutputData {
    /// Access to Winit window
    pub(crate) window: WinitWindow,
    /// GL surface of the window
    pub(crate) surface: Surface<WindowSurface>,
    /// Event listeners registered from the webview controller
    pub(crate) event_listeners: EventListeners,
    /// The mouse physical position in the web view.
    mouse_position: Cell<Option<PhysicalPosition<f64>>>,
    /// Modifiers state of the keyboard.
    modifiers_state: Cell<ModifiersState>,
    /// State to indicate if the window is resizing.
    pub(crate) resizing: bool,
    // TODO: These two fields should unified once we figure out servo's menu events.
    /// Context menu webview. This is only used in wayland currently.
    #[cfg(linux)]
    pub(crate) context_menu: Option<ContextMenu>,
    /// Global menu event receiver for muda crate
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) menu_event_receiver: MenuEventReceiver,
    /// Window tabs manager
    pub(crate) tab_manager: TabManager,
    pub(crate) focused_webview_id: Option<WebViewId>,
}
