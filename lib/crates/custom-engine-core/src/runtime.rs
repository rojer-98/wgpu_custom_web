use derive_more::Display;
use log::{debug, info};
use winit::window::Window;

use crate::{context::Context, errors::CoreError, worker::Worker};

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
    pub surface: &'a wgpu::Surface<'a>,
}

#[derive(Debug)]
pub struct Runtime<'a> {
    pub adapter_info: wgpu::AdapterInfo,
    pub adapter_features: wgpu::Features,
    pub power_preference: wgpu::PowerPreference,
    pub limits: wgpu::Limits,

    surface: Option<wgpu::Surface<'a>>,
    size: (u32, u32),
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> Runtime<'a> {
    pub async fn init(window: Option<&'a Window>) -> Result<Self, CoreError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = window.and_then(|w| instance.create_surface(w).ok());

        let power_preference = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(CoreError::RequestAdapter)?;

        let adapter_info = adapter.get_info();
        let adapter_features = adapter.features();

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
            window
                .map(|w| {
                    let i_s = w.inner_size();
                    (i_s.width, i_s.height)
                })
                .filter(|(w, h)| *w != 0 || *h != 0)
                .unwrap_or((1200, 1600))
        };

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

        Ok(Self {
            limits,
            power_preference,
            adapter_features,
            adapter_info,

            surface,
            size,

            adapter,
            queue,
            device,
        })
    }

    pub fn worker_texture(
        &self,
        texture_path: &str,
        texture_size: u32,
        image_format: ImageFormat,
    ) -> Result<Worker<'_>, CoreError> {
        Worker::new(
            (texture_size, texture_size),
            1.,
            RuntimeKind::Texture(texture_path.to_string(), image_format),
            &self.device,
            &self.queue,
            self.limits.clone(),
            None,
            Context::new(),
        )
    }

    pub fn worker_surface(&self) -> Result<Worker<'_>, CoreError> {
        Worker::new(
            self.size,
            1.,
            RuntimeKind::Winit(
                self.configure_surface()
                    .ok_or(CoreError::SurfaceNotConfigured)?,
            ),
            &self.device,
            &self.queue,
            self.limits.clone(),
            None,
            Context::new(),
        )
    }

    pub fn reinit_worker(
        &'a self,
        mut worker: Worker<'a>,
        into_runtime: IntoRuntime,
    ) -> Result<Worker<'a>, CoreError> {
        let new_size = match into_runtime {
            IntoRuntime::Window => {
                worker.runtime_kind = RuntimeKind::Winit(
                    self.configure_surface()
                        .ok_or(CoreError::SurfaceNotConfigured)?,
                );

                self.size
            }
            IntoRuntime::Texture {
                path,
                format,
                new_size,
            } => {
                worker.runtime_kind = RuntimeKind::Texture(path, format);
                new_size
            }
        };

        worker.init_with_size(new_size)?;

        Ok(worker)
    }

    fn configure_surface(&self) -> Option<SurfaceProperties> {
        let surface = self.surface.as_ref()?;
        let size = self.size;

        let surface_caps = surface.get_capabilities(&self.adapter);
        info!("Surface capabilities: {surface_caps:#?}");

        let limits = &self.limits;
        info!("Limits: {limits:#?}");

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .cloned()
            .unwrap_or(surface_caps.formats[0]);
        info!("Surface format: {surface_format:#?}");

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
        surface.configure(&self.device, &config);

        Some(SurfaceProperties { config, surface })
    }
}
