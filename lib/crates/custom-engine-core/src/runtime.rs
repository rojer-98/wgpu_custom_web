use std::time::Duration;

use derive_more::Display;
use log::{debug, error};
use pollster::block_on;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    context::Context,
    errors::CoreError,
    traits::{EventHandler, OnEvent, RenderWorker},
    worker::Worker,
};

#[derive(Debug, Display)]
pub enum ImageFormat {
    #[display(fmt = "png")]
    Png,
    #[display(fmt = "jpg")]
    Jpeg,
}

#[derive(Debug)]
pub(crate) struct SurfaceProperties<'a> {
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'a>,
}

pub struct Runtime<'a, R: RenderWorker + 'a, H: EventHandler<R>> {
    pub(crate) size: (u32, u32),
    pub(crate) limits: wgpu::Limits,
    pub(crate) instance: wgpu::Instance,
    pub(crate) power_preference: wgpu::PowerPreference,

    worker: Option<Worker<'a>>,
    render: R,
    handler: H,
}

impl<'a, E: OnEvent + 'static, R: RenderWorker + 'a, H: EventHandler<R>> ApplicationHandler<E>
    for Runtime<'a, R, H>
{
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: E) {
        event.on_event()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let w = self.worker.as_mut().unwrap();

        match self
            .render
            .update(w, &event, Duration::from_secs(1))
            .and(self.render.render(w))
        {
            Err(CoreError::SurfaceError(wgpu::SurfaceError::Lost)) => w.resize(),
            Err(CoreError::SurfaceError(wgpu::SurfaceError::Timeout)) => w.resize(),
            Err(CoreError::SurfaceError(wgpu::SurfaceError::OutOfMemory)) => {
                event_loop.exit();
            }
            Err(e) => error!("{e}"),
            _ => {}
        }

        match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                w.resize_by_size((width, height));

                if let Err(e) = self.render.resize(w) {
                    error!("{e}");
                }
            }
            WindowEvent::Focused(focused) => {
                if let Err(e) = self.handler.on_focused(
                    &mut self.render,
                    self.worker.as_mut().unwrap(),
                    focused,
                ) {
                    error!("{e}");
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                w.resize_by_scale(scale_factor);

                if let Err(e) = self.render.resize(w) {
                    error!("{e}");
                }
            }
            WindowEvent::ThemeChanged(theme) => {
                if let Err(e) = self.handler.on_theme(&mut self.render, w, theme) {
                    error!("{e}");
                }
            }
            WindowEvent::Occluded(occluded) => {
                if let Err(e) = self.handler.on_occluded(&mut self.render, w, occluded) {
                    error!("{e}");
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Err(e) = self
                    .handler
                    .on_modifiers_changed(&mut self.render, w, modifiers)
                {
                    error!("{e}");
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                if let Err(e) =
                    self.handler
                        .on_mouse_wheel(&mut self.render, w, device_id, delta, phase)
                {
                    error!("{e}");
                }
            }
            WindowEvent::KeyboardInput {
                is_synthetic,
                device_id,
                event,
            } => {
                if let Err(e) = self.handler.on_keyboard_input(
                    &mut self.render,
                    w,
                    device_id,
                    event,
                    is_synthetic,
                ) {
                    error!("{e}");
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if let Err(e) =
                    self.handler
                        .on_mouse_input(&mut self.render, w, device_id, state, button)
                {
                    error!("{e}");
                }
            }
            WindowEvent::CursorLeft { device_id } => {
                if let Err(e) = self.handler.on_cursor_left(&mut self.render, w, device_id) {
                    error!("{e}");
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Err(e) =
                    self.handler
                        .on_cursor_moved(&mut self.render, w, device_id, position)
                {
                    error!("{e}");
                }
            }
            WindowEvent::Ime(event) => {
                if let Err(e) = self.handler.on_ime(&mut self.render, w, event) {
                    error!("{e}");
                }
            }
            WindowEvent::PinchGesture {
                device_id,
                delta,
                phase,
            } => {
                if let Err(e) =
                    self.handler
                        .on_pinch_gesture(&mut self.render, w, device_id, delta, phase)
                {
                    error!("{e}");
                }
            }
            WindowEvent::RotationGesture {
                device_id,
                delta,
                phase,
            } => {
                if let Err(e) =
                    self.handler
                        .on_rotation_gesture(&mut self.render, w, device_id, delta, phase)
                {
                    error!("{e}");
                }
            }
            WindowEvent::PanGesture {
                device_id,
                delta,
                phase,
            } => {
                if let Err(e) =
                    self.handler
                        .on_pan_gesture(&mut self.render, w, device_id, delta, phase)
                {
                    error!("{e}");
                }
            }
            WindowEvent::DoubleTapGesture { device_id } => {
                if let Err(e) = self
                    .handler
                    .on_double_tap_gesture(&mut self.render, w, device_id)
                {
                    error!("{e}");
                }
            }
            WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => {
                if let Err(e) = self.handler.on_touchpad_pressure(
                    &mut self.render,
                    w,
                    device_id,
                    pressure,
                    stage,
                ) {
                    error!("{e}");
                }
            }
            WindowEvent::HoveredFileCancelled => {
                if let Err(e) = self.handler.on_hovered_file_cancelled(&mut self.render, w) {
                    error!("{e}");
                }
            }
            WindowEvent::CursorEntered { device_id } => {
                if let Err(e) = self
                    .handler
                    .on_cursor_entered(&mut self.render, w, device_id)
                {
                    error!("{e}");
                }
            }
            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {
                if let Err(e) =
                    self.handler
                        .on_axis_motion(&mut self.render, w, device_id, axis, value)
                {
                    error!("{e}");
                }
            }
            WindowEvent::DroppedFile(pb) => {
                if let Err(e) = self.handler.on_dropped_file(&mut self.render, w, pb) {
                    error!("{e}");
                }
            }
            WindowEvent::HoveredFile(pb) => {
                if let Err(e) = self.handler.on_hovered_file(&mut self.render, w, pb) {
                    error!("{e}");
                }
            }
            WindowEvent::Destroyed => {
                if let Err(e) = self.handler.on_destroyed(&mut self.render, w) {
                    error!("{e}");
                }
            }
            WindowEvent::Touch(t) => {
                if let Err(e) = self.handler.on_touch(&mut self.render, w, t) {
                    error!("{e}");
                }
            }
            WindowEvent::Moved(pp) => {
                if let Err(e) = self.handler.on_moved(&mut self.render, w, pp) {
                    error!("{e}");
                }
            }

            WindowEvent::ActivationTokenDone { .. } | WindowEvent::RedrawRequested => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Panic if window is not init
        let w = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(PhysicalSize::new(self.size.0, self.size.1)),
            )
            .unwrap();

        if let Err(e) = self.worker_init(w) {
            error!("{e}");
            return;
        }

        // Worker is presented always at this step
        let worker = self.worker.as_mut().unwrap();
        if let Err(e) = self.render.init(worker) {
            error!("{e}");
        }
    }
}

impl<'a, R: RenderWorker + 'a, H: EventHandler<R>> Runtime<'a, R, H> {
    pub fn new(size: (u32, u32)) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let power_preference = wgpu::PowerPreference::default();
        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        Self {
            instance,
            power_preference,
            limits,
            size,
            render: R::new(),
            handler: H::default(),
            worker: None,
        }
    }

    // Create only in winit context
    fn worker_init(&mut self, window: Window) -> Result<(), CoreError> {
        let Self {
            limits,
            instance,
            power_preference,
            worker,
            ..
        } = self;

        let size = if cfg!(target_arch = "wasm32") {
            (
                limits.max_texture_dimension_2d,
                limits.max_texture_dimension_2d,
            )
        } else {
            let i_s = window.inner_size();

            if i_s.height == 0 || i_s.width == 0 {
                self.size
            } else {
                (i_s.width, i_s.height)
            }
        };

        let surface = instance.create_surface(window)?;
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: *power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or(CoreError::RequestAdapter)?;
        let adapter_info = adapter.get_info();
        let adapter_features = adapter.features();

        debug!(
            "
Adapter: 
    Info: {adapter_info:#?},
    Features: {adapter_features:#?},
    Limits: {limits:#?}"
        );

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: adapter_features,
                required_limits: limits.clone(),
                label: None,
            },
            None,
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .cloned()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 0,
        };
        surface.configure(&device, &config);

        *worker = Some(Worker::new(
            size,
            1.,
            SurfaceProperties { config, surface },
            device,
            queue,
            limits.clone(),
            None,
            Context::new(),
        )?);

        Ok(())
    }
}
