pub mod foreign;
pub mod key_bindings;

use std::{collections::HashMap, mem, num::NonZeroU32, sync::Arc};

use anyhow::Result;
use instant::{Duration, Instant};
use log::{error, info};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, DeviceId, ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{
        Cursor, CursorGrabMode, CursorIcon, CustomCursor, CustomCursorSource, Fullscreen,
        ResizeDirection, Theme, Window, WindowId,
    },
};

use custom_engine_core::runtime::Runtime;

use crate::{application::foreign::UserEvent, config::EngineConfig};

#[derive(Debug)]
pub enum ClickType {
    Simple,
    Double,
    Triple,
}

#[derive(Debug)]
pub struct AppState {
    pub(crate) cursor_position: PhysicalPosition<f64>,

    pub(crate) click_state: ElementState,
    pub(crate) click_position: PhysicalPosition<f64>,

    pub(crate) last_click_timestamp: Option<Instant>,
    is_double_click: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(0., 0.),
            click_state: ElementState::Released,
            click_position: PhysicalPosition::new(0., 0.),
            last_click_timestamp: None,
            is_double_click: false,
        }
    }

    pub fn clicked(&mut self) {
        if self.last_click_timestamp.is_none() {
            self.last_click_timestamp = Some(Instant::now());
        } else {
            let lct = self.last_click_timestamp.take().unwrap();
            let elapsed = lct.elapsed();

            self.is_double_click = elapsed.saturating_sub(Duration::from_millis(400)).is_zero();
        }

        self.click_position = self.cursor_position;
    }

    pub fn is_double_click(&mut self) -> bool {
        self.is_double_click
    }
}

#[derive(Debug)]
pub struct App<'a> {
    runtime: Runtime<'a>,

    config: EngineConfig,
    state: AppState,
    windows: HashMap<WindowId, WindowState>,
}

impl<'a> App<'a> {
    fn new(runtime: Runtime<'a>, config: EngineConfig, event_loop: &EventLoop<UserEvent>) -> Self {
        Self {
            state: AppState::new(),
            windows: Default::default(),
            config,
            runtime,
        }
    }

    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        _tab_id: Option<String>,
    ) -> Result<WindowId> {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("Wgpu custom engine")
            .with_transparent(true);

        #[cfg(any(x11_platform, wayland_platform))]
        if let Some(token) = event_loop.read_token_from_env() {
            startup_notify::reset_activation_token_env();
            info!("Using token {:?} to activate a window", token);
            window_attributes = window_attributes.with_activation_token(token);
        }

        #[cfg(macos_platform)]
        if let Some(tab_id) = _tab_id {
            window_attributes = window_attributes.with_tabbing_identifier(&tab_id);
        }

        #[cfg(web_platform)]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes = window_attributes.with_append(true);
        }

        let window = event_loop.create_window(window_attributes)?;

        #[cfg(ios_platform)]
        {
            use winit::platform::ios::WindowExtIOS;
            window.recognize_doubletap_gesture(true);
            window.recognize_pinch_gesture(true);
            window.recognize_rotation_gesture(true);
            window.recognize_pan_gesture(true, 2, 2);
        }

        let window_state = WindowState::new(self, window)?;
        let window_id = window_state.window.id();
        info!("Created new window with id={window_id:?}");

        self.windows.insert(window_id, window_state);

        Ok(window_id)
    }

    fn handle_action(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, action: Action) {
        // let cursor_position = self.cursor_position;
        let window = self.windows.get_mut(&window_id).unwrap();
        info!("Executing action: {action:?}");
        match action {
            Action::CloseWindow => {
                let _ = self.windows.remove(&window_id);
            }
            Action::CreateNewWindow => {
                #[cfg(any(x11_platform, wayland_platform))]
                if let Err(err) = window.window.request_activation_token() {
                    info!("Failed to get activation token: {err}");
                } else {
                    return;
                }

                if let Err(err) = self.create_window(event_loop, None) {
                    error!("Error creating new window: {err}");
                }
            }
            Action::ToggleResizeIncrements => window.toggle_resize_increments(),
            Action::ToggleCursorVisibility => window.toggle_cursor_visibility(),
            Action::ToggleResizable => window.toggle_resizable(),
            Action::ToggleDecorations => window.toggle_decorations(),
            Action::ToggleFullscreen => window.toggle_fullscreen(),
            Action::ToggleMaximize => window.toggle_maximize(),
            Action::ToggleImeInput => window.toggle_ime(),
            Action::Minimize => window.minimize(),
            Action::NextCursor => window.next_cursor(),
            Action::NextCustomCursor => window.next_custom_cursor(&self.custom_cursors),
            #[cfg(web_platform)]
            Action::UrlCustomCursor => window.url_custom_cursor(event_loop),
            #[cfg(web_platform)]
            Action::AnimationCustomCursor => {
                window.animation_custom_cursor(event_loop, &self.custom_cursors)
            }
            Action::CycleCursorGrab => window.cycle_cursor_grab(),
            Action::DragWindow => window.drag_window(),
            Action::DragResizeWindow => window.drag_resize_window(),
            Action::ShowWindowMenu => window.show_menu(),
            Action::PrintHelp => self.print_help(),
            #[cfg(macos_platform)]
            Action::CycleOptionAsAlt => window.cycle_option_as_alt(),
            #[cfg(macos_platform)]
            Action::CreateNewTab => {
                let tab_id = window.window.tabbing_identifier();
                if let Err(err) = self.create_window(event_loop, Some(tab_id)) {
                    error!("Error creating new window: {err}");
                }
            }
            Action::RequestResize => window.swap_dimensions(),
        }
    }

    fn dump_monitors(&self, event_loop: &ActiveEventLoop) {
        info!("Monitors information");
        let primary_monitor = event_loop.primary_monitor();
        for monitor in event_loop.available_monitors() {
            let intro = if primary_monitor.as_ref() == Some(&monitor) {
                "Primary monitor"
            } else {
                "Monitor"
            };

            if let Some(name) = monitor.name() {
                info!("{intro}: {name}");
            } else {
                info!("{intro}: [no name]");
            }

            let PhysicalSize { width, height } = monitor.size();
            info!(
                "  Current mode: {width}x{height}{}",
                if let Some(m_hz) = monitor.refresh_rate_millihertz() {
                    format!(" @ {}.{} Hz", m_hz / 1000, m_hz % 1000)
                } else {
                    String::new()
                }
            );

            let PhysicalPosition { x, y } = monitor.position();
            info!("  Position: {x},{y}");

            info!("  Scale factor: {}", monitor.scale_factor());

            info!("  Available modes (width x height x bit-depth):");
            for mode in monitor.video_modes() {
                let PhysicalSize { width, height } = mode.size();
                let bits = mode.bit_depth();
                let m_hz = mode.refresh_rate_millihertz();
                info!(
                    "    {width}x{height}x{bits} @ {}.{} Hz",
                    m_hz / 1000,
                    m_hz % 1000
                );
            }
        }
    }

    /// Process the key binding.
    fn process_key_binding(key: &str, mods: &ModifiersState) -> Option<Action> {
        KEY_BINDINGS.iter().find_map(|binding| {
            binding
                .is_triggered_by(&key, mods)
                .then_some(binding.action)
        })
    }

    /// Process mouse binding.
    fn process_mouse_binding(button: MouseButton, mods: &ModifiersState) -> Option<Action> {
        MOUSE_BINDINGS.iter().find_map(|binding| {
            binding
                .is_triggered_by(&button, mods)
                .then_some(binding.action)
        })
    }

    fn print_help(&self) {
        info!("Keyboard bindings:");
        for binding in KEY_BINDINGS {
            info!(
                "{}{:<10} - {} ({})",
                modifiers_to_string(binding.mods),
                binding.trigger,
                binding.action,
                binding.action.help(),
            );
        }
        info!("Mouse bindings:");
        for binding in MOUSE_BINDINGS {
            info!(
                "{}{:<10} - {} ({})",
                modifiers_to_string(binding.mods),
                mouse_button_to_string(binding.trigger),
                binding.action,
                binding.action.help(),
            );
        }
    }
}

impl<'a> ApplicationHandler<UserEvent> for App<'a> {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        event.on_event()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        //    r.update(&mut worker_surface, event, Duration::from_secs(1))
        //       .unwrap();

        //   match event {
        //       // Worker
        //       WindowEvent::RedrawRequested => {
        //           match block_on(async { r.render(&mut worker_surface).await }) {
        //               Err(CoreError::SurfaceError(wgpu::SurfaceError::Lost)) => {
        //                   worker_surface.resize()
        //               }
        //               Err(CoreError::SurfaceError(wgpu::SurfaceError::Timeout)) => {
        //                   worker_surface.resize()
        //               }
        //               Err(CoreError::SurfaceError(wgpu::SurfaceError::OutOfMemory)) => {
        //                   control_flow.exit()
        //               }
        //               Err(e) => error!("{e}"),
        //               _ => {}
        //           }

        //           window.request_redraw();
        //       }
        //       WindowEvent::Resized(new_size) => {
        //           worker_surface.resize_by_size((new_size.width, new_size.height));
        //           r.resize(&mut worker_surface).unwrap();
        //       }
        //       WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        //           worker_surface.resize_by_scale(*scale_factor);
        //           r.resize(&mut worker_surface).unwrap();
        //       }

        //       // Mouse
        //       // WindowEvent::CursorMoved { position, .. } => {
        //       //     if app_state.click_state.is_pressed() {
        //       //         let diff = (
        //       //             position.x - app_state.cursor_position.x,
        //       //             position.y - app_state.cursor_position.y,
        //       //         );
        //       //         r.move_to(&mut worker_surface, diff).unwrap();
        //       //     }

        //       //     app_state.cursor_position = *position;
        //       // }
        //       // WindowEvent::MouseInput { state, .. } => {
        //       //     if state.is_pressed() {
        //       //         app_state.clicked();

        //       //         r.click(&mut worker_surface, &app_state).unwrap();
        //       //     }

        //       //     app_state.click_state = *state;
        //       // }

        //       // Exit
        //       WindowEvent::CloseRequested
        //       | WindowEvent::KeyboardInput {
        //           event:
        //               KeyEvent {
        //                   logical_key: Key::Named(NamedKey::Escape),
        //                   ..
        //               },
        //           ..
        //       } => {
        //           control_flow.exit();
        //       }
        //       _ => {}
        //   }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {}
}

/// State of the window.
#[derive(Debug)]
struct WindowState {
    ime: bool,
    window: Arc<Window>,
    theme: Theme,
    cursor_position: Option<PhysicalPosition<f64>>,
    modifiers: ModifiersState,
    occluded: bool,
    cursor_grab: CursorGrabMode,
    zoom: f64,
    rotated: f32,
    panned: PhysicalPosition<f32>,

    // Cursor states.
    named_idx: usize,
    custom_idx: usize,
    cursor_hidden: bool,
}

impl WindowState {
    fn new(app: &App, window: Window) -> Result<Self> {
        let window = Arc::new(window);

        let theme = window.theme().unwrap_or(Theme::Dark);
        info!("Theme: {theme:?}");
        let named_idx = 0;
        window.set_cursor(CURSORS[named_idx]);

        // Allow IME out of the box.
        let ime = true;
        window.set_ime_allowed(ime);

        let size = window.inner_size();
        let mut state = Self {
            custom_idx: app.custom_cursors.len() - 1,
            cursor_grab: CursorGrabMode::None,
            named_idx,
            window,
            theme,
            ime,
            cursor_position: Default::default(),
            cursor_hidden: Default::default(),
            modifiers: Default::default(),
            occluded: Default::default(),
            rotated: Default::default(),
            panned: Default::default(),
            zoom: Default::default(),
        };

        state.resize(size);
        Ok(state)
    }

    pub fn toggle_ime(&mut self) {
        self.ime = !self.ime;
        self.window.set_ime_allowed(self.ime);
        if let Some(position) = self.ime.then_some(self.cursor_position).flatten() {
            self.window
                .set_ime_cursor_area(position, PhysicalSize::new(20, 20));
        }
    }

    pub fn minimize(&mut self) {
        self.window.set_minimized(true);
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = Some(position);
        if self.ime {
            self.window
                .set_ime_cursor_area(position, PhysicalSize::new(20, 20));
        }
    }

    pub fn cursor_left(&mut self) {
        self.cursor_position = None;
    }

    /// Toggle maximized.
    fn toggle_maximize(&self) {
        let maximized = self.window.is_maximized();
        self.window.set_maximized(!maximized);
    }

    /// Toggle window decorations.
    fn toggle_decorations(&self) {
        let decorated = self.window.is_decorated();
        self.window.set_decorations(!decorated);
    }

    /// Toggle window resizable state.
    fn toggle_resizable(&self) {
        let resizable = self.window.is_resizable();
        self.window.set_resizable(!resizable);
    }

    /// Toggle cursor visibility
    fn toggle_cursor_visibility(&mut self) {
        self.cursor_hidden = !self.cursor_hidden;
        self.window.set_cursor_visible(!self.cursor_hidden);
    }

    /// Toggle resize increments on a window.
    fn toggle_resize_increments(&mut self) {
        let new_increments = match self.window.resize_increments() {
            Some(_) => None,
            None => Some(LogicalSize::new(25.0, 25.0)),
        };
        info!("Had increments: {}", new_increments.is_none());
        self.window.set_resize_increments(new_increments);
    }

    /// Toggle fullscreen.
    fn toggle_fullscreen(&self) {
        let fullscreen = if self.window.fullscreen().is_some() {
            None
        } else {
            Some(Fullscreen::Borderless(None))
        };

        self.window.set_fullscreen(fullscreen);
    }

    /// Cycle through the grab modes ignoring errors.
    fn cycle_cursor_grab(&mut self) {
        self.cursor_grab = match self.cursor_grab {
            CursorGrabMode::None => CursorGrabMode::Confined,
            CursorGrabMode::Confined => CursorGrabMode::Locked,
            CursorGrabMode::Locked => CursorGrabMode::None,
        };
        info!("Changing cursor grab mode to {:?}", self.cursor_grab);
        if let Err(err) = self.window.set_cursor_grab(self.cursor_grab) {
            error!("Error setting cursor grab: {err}");
        }
    }

    #[cfg(macos_platform)]
    fn cycle_option_as_alt(&mut self) {
        self.option_as_alt = match self.option_as_alt {
            OptionAsAlt::None => OptionAsAlt::OnlyLeft,
            OptionAsAlt::OnlyLeft => OptionAsAlt::OnlyRight,
            OptionAsAlt::OnlyRight => OptionAsAlt::Both,
            OptionAsAlt::Both => OptionAsAlt::None,
        };
        info!("Setting option as alt {:?}", self.option_as_alt);
        self.window.set_option_as_alt(self.option_as_alt);
    }

    /// Swap the window dimensions with `request_inner_size`.
    fn swap_dimensions(&mut self) {
        let old_inner_size = self.window.inner_size();
        let mut inner_size = old_inner_size;

        mem::swap(&mut inner_size.width, &mut inner_size.height);
        info!("Requesting resize from {old_inner_size:?} to {inner_size:?}");

        if let Some(new_inner_size) = self.window.request_inner_size(inner_size) {
            if old_inner_size == new_inner_size {
                info!("Inner size change got ignored");
            } else {
                self.resize(new_inner_size);
            }
        } else {
            info!("Request inner size is asynchronous");
        }
    }

    /// Pick the next cursor.
    fn next_cursor(&mut self) {
        self.named_idx = (self.named_idx + 1) % CURSORS.len();
        info!("Setting cursor to \"{:?}\"", CURSORS[self.named_idx]);
        self.window
            .set_cursor(Cursor::Icon(CURSORS[self.named_idx]));
    }

    /// Pick the next custom cursor.
    fn next_custom_cursor(&mut self, custom_cursors: &[CustomCursor]) {
        self.custom_idx = (self.custom_idx + 1) % custom_cursors.len();
        let cursor = Cursor::Custom(custom_cursors[self.custom_idx].clone());
        self.window.set_cursor(cursor);
    }

    /// Custom cursor from an URL.
    #[cfg(web_platform)]
    fn url_custom_cursor(&mut self, event_loop: &ActiveEventLoop) {
        let cursor = event_loop.create_custom_cursor(url_custom_cursor());

        self.window.set_cursor(cursor);
    }

    /// Custom cursor from a URL.
    #[cfg(web_platform)]
    fn animation_custom_cursor(
        &mut self,
        event_loop: &ActiveEventLoop,
        custom_cursors: &[CustomCursor],
    ) {
        use std::time::Duration;
        use winit::platform::web::CustomCursorExtWebSys;

        let cursors = vec![
            custom_cursors[0].clone(),
            custom_cursors[1].clone(),
            event_loop.create_custom_cursor(url_custom_cursor()),
        ];
        let cursor = CustomCursor::from_animation(Duration::from_secs(3), cursors).unwrap();
        let cursor = event_loop.create_custom_cursor(cursor);

        self.window.set_cursor(cursor);
    }

    /// Resize the window to the new size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        info!("Resized to {size:?}");
        #[cfg(not(any(android_platform, ios_platform)))]
        {
            let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
            {
                (Some(width), Some(height)) => (width, height),
                _ => return,
            };
            self.surface
                .resize(width, height)
                .expect("failed to resize inner buffer");
        }
        self.window.request_redraw();
    }

    /// Change the theme.
    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.window.request_redraw();
    }

    /// Show window menu.
    fn show_menu(&self) {
        if let Some(position) = self.cursor_position {
            self.window.show_window_menu(position);
        }
    }

    /// Drag the window.
    fn drag_window(&self) {
        if let Err(err) = self.window.drag_window() {
            info!("Error starting window drag: {err}");
        } else {
            info!("Dragging window Window={:?}", self.window.id());
        }
    }

    /// Drag-resize the window.
    fn drag_resize_window(&self) {
        let position = match self.cursor_position {
            Some(position) => position,
            None => {
                info!("Drag-resize requires cursor to be inside the window");
                return;
            }
        };

        let win_size = self.window.inner_size();
        let border_size = BORDER_SIZE * self.window.scale_factor();

        let x_direction = if position.x < border_size {
            ResizeDirection::West
        } else if position.x > (win_size.width as f64 - border_size) {
            ResizeDirection::East
        } else {
            // Use arbitrary direction instead of None for simplicity.
            ResizeDirection::SouthEast
        };

        let y_direction = if position.y < border_size {
            ResizeDirection::North
        } else if position.y > (win_size.height as f64 - border_size) {
            ResizeDirection::South
        } else {
            // Use arbitrary direction instead of None for simplicity.
            ResizeDirection::SouthEast
        };

        let direction = match (x_direction, y_direction) {
            (ResizeDirection::West, ResizeDirection::North) => ResizeDirection::NorthWest,
            (ResizeDirection::West, ResizeDirection::South) => ResizeDirection::SouthWest,
            (ResizeDirection::West, _) => ResizeDirection::West,
            (ResizeDirection::East, ResizeDirection::North) => ResizeDirection::NorthEast,
            (ResizeDirection::East, ResizeDirection::South) => ResizeDirection::SouthEast,
            (ResizeDirection::East, _) => ResizeDirection::East,
            (_, ResizeDirection::South) => ResizeDirection::South,
            (_, ResizeDirection::North) => ResizeDirection::North,
            _ => return,
        };

        if let Err(err) = self.window.drag_resize_window(direction) {
            info!("Error starting window drag-resize: {err}");
        } else {
            info!("Drag-resizing window Window={:?}", self.window.id());
        }
    }

    /// Change window occlusion state.
    fn set_occluded(&mut self, occluded: bool) {
        self.occluded = occluded;
        if !occluded {
            self.window.request_redraw();
        }
    }

    /// Draw the window contents.
    fn draw(&mut self) -> Result<()> {
        if self.occluded {
            info!("Skipping drawing occluded window={:?}", self.window.id());
            return Ok(());
        }

        const WHITE: u32 = 0xffffffff;
        const DARK_GRAY: u32 = 0xff181818;

        let color = match self.theme {
            Theme::Light => WHITE,
            Theme::Dark => DARK_GRAY,
        };

        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(color);
        self.window.pre_present_notify();
        buffer.present()?;
        Ok(())
    }
}

struct Binding<T: Eq> {
    trigger: T,
    mods: ModifiersState,
    action: Action,
}

impl<T: Eq> Binding<T> {
    const fn new(trigger: T, mods: ModifiersState, action: Action) -> Self {
        Self {
            trigger,
            mods,
            action,
        }
    }

    fn is_triggered_by(&self, trigger: &T, mods: &ModifiersState) -> bool {
        &self.trigger == trigger && &self.mods == mods
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    CloseWindow,
    ToggleCursorVisibility,
    CreateNewWindow,
    ToggleResizeIncrements,
    ToggleImeInput,
    ToggleDecorations,
    ToggleResizable,
    ToggleFullscreen,
    ToggleMaximize,
    Minimize,
    NextCursor,
    NextCustomCursor,
    #[cfg(web_platform)]
    UrlCustomCursor,
    #[cfg(web_platform)]
    AnimationCustomCursor,
    CycleCursorGrab,
    PrintHelp,
    DragWindow,
    DragResizeWindow,
    ShowWindowMenu,
    #[cfg(macos_platform)]
    CycleOptionAsAlt,
    #[cfg(macos_platform)]
    CreateNewTab,
    RequestResize,
}

impl Action {
    fn help(&self) -> &'static str {
        match self {
            Action::CloseWindow => "Close window",
            Action::ToggleCursorVisibility => "Hide cursor",
            Action::CreateNewWindow => "Create new window",
            Action::ToggleImeInput => "Toggle IME input",
            Action::ToggleDecorations => "Toggle decorations",
            Action::ToggleResizable => "Toggle window resizable state",
            Action::ToggleFullscreen => "Toggle fullscreen",
            Action::ToggleMaximize => "Maximize",
            Action::Minimize => "Minimize",
            Action::ToggleResizeIncrements => "Use resize increments when resizing window",
            Action::NextCursor => "Advance the cursor to the next value",
            Action::NextCustomCursor => "Advance custom cursor to the next value",
            #[cfg(web_platform)]
            Action::UrlCustomCursor => "Custom cursor from an URL",
            #[cfg(web_platform)]
            Action::AnimationCustomCursor => "Custom cursor from an animation",
            Action::CycleCursorGrab => "Cycle through cursor grab mode",
            Action::PrintHelp => "Print help",
            Action::DragWindow => "Start window drag",
            Action::DragResizeWindow => "Start window drag-resize",
            Action::ShowWindowMenu => "Show window menu",
            #[cfg(macos_platform)]
            Action::CycleOptionAsAlt => "Cycle option as alt mode",
            #[cfg(macos_platform)]
            Action::CreateNewTab => "Create new tab",
            Action::RequestResize => "Request a resize",
        }
    }
}

fn decode_cursor(bytes: &[u8]) -> CustomCursorSource {
    let img = image::load_from_memory(bytes).unwrap().to_rgba8();
    let samples = img.into_flat_samples();
    let (_, w, h) = samples.extents();
    let (w, h) = (w as u16, h as u16);
    CustomCursor::from_rgba(samples.samples, w, h, w / 2, h / 2).unwrap()
}

#[cfg(web_platform)]
fn url_custom_cursor() -> CustomCursorSource {
    use std::sync::atomic::{AtomicU64, Ordering};

    use winit::platform::web::CustomCursorExtWebSys;

    static URL_COUNTER: AtomicU64 = AtomicU64::new(0);

    CustomCursor::from_url(
        format!(
            "https://picsum.photos/128?random={}",
            URL_COUNTER.fetch_add(1, Ordering::Relaxed)
        ),
        64,
        64,
    )
}

fn modifiers_to_string(mods: ModifiersState) -> String {
    let mut mods_line = String::new();
    // Always add + since it's printed as a part of the bindings.
    for (modifier, desc) in [
        (ModifiersState::SUPER, "Super+"),
        (ModifiersState::ALT, "Alt+"),
        (ModifiersState::CONTROL, "Ctrl+"),
        (ModifiersState::SHIFT, "Shift+"),
    ] {
        if !mods.contains(modifier) {
            continue;
        }

        mods_line.push_str(desc);
    }
    mods_line
}

fn mouse_button_to_string(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "LMB",
        MouseButton::Right => "RMB",
        MouseButton::Middle => "MMB",
        MouseButton::Back => "Back",
        MouseButton::Forward => "Forward",
        MouseButton::Other(_) => "",
    }
}

/// Cursor list to cycle through.
const CURSORS: &[CursorIcon] = &[
    CursorIcon::Default,
    CursorIcon::Crosshair,
    CursorIcon::Pointer,
    CursorIcon::Move,
    CursorIcon::Text,
    CursorIcon::Wait,
    CursorIcon::Help,
    CursorIcon::Progress,
    CursorIcon::NotAllowed,
    CursorIcon::ContextMenu,
    CursorIcon::Cell,
    CursorIcon::VerticalText,
    CursorIcon::Alias,
    CursorIcon::Copy,
    CursorIcon::NoDrop,
    CursorIcon::Grab,
    CursorIcon::Grabbing,
    CursorIcon::AllScroll,
    CursorIcon::ZoomIn,
    CursorIcon::ZoomOut,
    CursorIcon::EResize,
    CursorIcon::NResize,
    CursorIcon::NeResize,
    CursorIcon::NwResize,
    CursorIcon::SResize,
    CursorIcon::SeResize,
    CursorIcon::SwResize,
    CursorIcon::WResize,
    CursorIcon::EwResize,
    CursorIcon::NsResize,
    CursorIcon::NeswResize,
    CursorIcon::NwseResize,
    CursorIcon::ColResize,
    CursorIcon::RowResize,
];

const KEY_BINDINGS: &[Binding<&'static str>] = &[
    Binding::new("Q", ModifiersState::CONTROL, Action::CloseWindow),
    Binding::new("H", ModifiersState::CONTROL, Action::PrintHelp),
    Binding::new("F", ModifiersState::CONTROL, Action::ToggleFullscreen),
    Binding::new("D", ModifiersState::CONTROL, Action::ToggleDecorations),
    Binding::new("I", ModifiersState::CONTROL, Action::ToggleImeInput),
    Binding::new("L", ModifiersState::CONTROL, Action::CycleCursorGrab),
    Binding::new("P", ModifiersState::CONTROL, Action::ToggleResizeIncrements),
    Binding::new("R", ModifiersState::CONTROL, Action::ToggleResizable),
    Binding::new("R", ModifiersState::ALT, Action::RequestResize),
    // M.
    Binding::new("M", ModifiersState::CONTROL, Action::ToggleMaximize),
    Binding::new("M", ModifiersState::ALT, Action::Minimize),
    // N.
    Binding::new("N", ModifiersState::CONTROL, Action::CreateNewWindow),
    // C.
    Binding::new("C", ModifiersState::CONTROL, Action::NextCursor),
    Binding::new("C", ModifiersState::ALT, Action::NextCustomCursor),
    #[cfg(web_platform)]
    Binding::new(
        "C",
        ModifiersState::CONTROL.union(ModifiersState::SHIFT),
        Action::UrlCustomCursor,
    ),
    #[cfg(web_platform)]
    Binding::new(
        "C",
        ModifiersState::ALT.union(ModifiersState::SHIFT),
        Action::AnimationCustomCursor,
    ),
    Binding::new("Z", ModifiersState::CONTROL, Action::ToggleCursorVisibility),
    #[cfg(macos_platform)]
    Binding::new("T", ModifiersState::SUPER, Action::CreateNewTab),
    #[cfg(macos_platform)]
    Binding::new("O", ModifiersState::CONTROL, Action::CycleOptionAsAlt),
];

const MOUSE_BINDINGS: &[Binding<MouseButton>] = &[
    Binding::new(
        MouseButton::Left,
        ModifiersState::ALT,
        Action::DragResizeWindow,
    ),
    Binding::new(
        MouseButton::Left,
        ModifiersState::CONTROL,
        Action::DragWindow,
    ),
    Binding::new(
        MouseButton::Right,
        ModifiersState::CONTROL,
        Action::ShowWindowMenu,
    ),
];
