use core::panic;
use std::{collections::HashMap, time::Duration};

use derive_more::Display;
use log::{debug, error, info};
use pollster::block_on;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{self, Window, WindowId},
};

use crate::{
    context::Context,
    errors::CoreError,
    traits::{OnEvent, RenderWorker},
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
pub enum IntoRuntime {
    Window,
    Texture {
        path: String,
        format: ImageFormat,
        new_size: (u32, u32),
    },
}

#[derive(Debug)]
pub(crate) enum RuntimeKind<'a> {
    Winit(SurfaceProperties<'a>),
    Texture(String, ImageFormat),
}

#[derive(Debug)]
pub(crate) struct SurfaceProperties<'a> {
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'a>,
}

pub struct Runtime<'a> {
    pub size: (u32, u32),
    //pub state: RuntimeState,
    workers: HashMap<WindowId, Worker<'a>>,
    renders: Vec<Box<dyn RenderWorker + 'a>>,
}

/*
Event::WindowEvent {
    ref event,
    window_id,
} if window_id == window.id() => {

    match event {

        // Mouse
        // WindowEvent::CursorMoved { position, .. } => {
        //     if app_state.click_state.is_pressed() {
        //         let diff = (
        //             position.x - app_state.cursor_position.x,
        //             position.y - app_state.cursor_position.y,
        //         );
        //         r.move_to(&mut worker_surface, diff).unwrap();
        //     }

        //     app_state.cursor_position = *position;
        // }
        // WindowEvent::MouseInput { state, .. } => {
        //     if state.is_pressed() {
        //         app_state.clicked();

        //         r.click(&mut worker_surface, &app_state).unwrap();
        //     }

        //     app_state.click_state = *state;
        // }
        _ => {}
    }
}

*/

impl<'a, E: OnEvent + 'static> ApplicationHandler<E> for Runtime<'a> {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        event.on_event()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let worker = self.workers.get_mut(&window_id);

        match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                if let Some(w) = worker {
                    w.resize_by_size((width, height));
                    self.renders.iter_mut().for_each(|r| {
                        if let Err(e) = r.resize(w) {
                            error!("{e}");
                        }
                    });
                }
            }
            WindowEvent::Focused(_focused) => {}
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(w) = worker {
                    w.resize_by_scale(scale_factor);
                    self.renders.iter_mut().for_each(|r| {
                        if let Err(e) = r.resize(w) {
                            error!("{e}");
                        }
                    });
                }
            }
            WindowEvent::ThemeChanged(_theme) => {}
            WindowEvent::RedrawRequested => {
                if let Some(w) = worker {
                    self.renders.iter_mut().for_each(|r| {
                        match r.update(w, &event, Duration::from_secs(1)).and(r.render(w)) {
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::Lost)) => w.resize(),
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::Timeout)) => w.resize(),
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::OutOfMemory)) => {
                                event_loop.exit();
                            }
                            Err(e) => error!("{e}"),
                            _ => {}
                        }
                    });

                    //window.request_redraw();
                }
            }
            WindowEvent::Occluded(_occluded) => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(_modifiers) => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::KeyboardInput {
                is_synthetic: false,
                ..
            } => {}
            WindowEvent::MouseInput { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::CursorMoved { .. } => {}
            WindowEvent::ActivationTokenDone { token: _token, .. } => {
                match event_loop.create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(self.size.0, self.size.1)),
                ) {
                    Ok(w) => {
                        if let Err(e) = block_on(async { self.worker_surface(w).await }) {
                            error!("{e}");
                        }
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            WindowEvent::Ime(event) => {}
            WindowEvent::PinchGesture { delta, .. } => {}
            WindowEvent::RotationGesture { delta, .. } => {}
            WindowEvent::PanGesture { delta, phase, .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. }
            | WindowEvent::HoveredFileCancelled
            | WindowEvent::KeyboardInput { .. }
            | WindowEvent::CursorEntered { .. }
            | WindowEvent::AxisMotion { .. }
            | WindowEvent::DroppedFile(_)
            | WindowEvent::HoveredFile(_)
            | WindowEvent::Destroyed
            | WindowEvent::Touch(_)
            | WindowEvent::Moved(_) => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}
}

impl<'a> Runtime<'a> {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            size,
            workers: HashMap::default(),
            renders: Vec::default(),
        }
    }

    async fn worker_surface(&mut self, window: Window) -> Result<(), CoreError> {
        let window_id = window.id();
        let power_preference = wgpu::PowerPreference::default();
        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let size = if cfg!(target_arch = "wasm32") {
            (
                limits.max_texture_dimension_2d,
                limits.max_texture_dimension_2d,
            )
        } else {
            let i_s = window.inner_size();

            (i_s.width, i_s.height)
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
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

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: adapter_features,
                    required_limits: limits.clone(),
                    label: None,
                },
                None,
            )
            .await?;

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

        self.workers.insert(
            window_id,
            Worker::new(
                size,
                1.,
                RuntimeKind::Winit(SurfaceProperties { config, surface }),
                device,
                queue,
                limits,
                None,
                Context::new(),
            )?,
        );

        Ok(())
    }
}
