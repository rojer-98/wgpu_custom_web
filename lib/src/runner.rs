use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use log::LevelFilter;
use log::{error, info};
#[cfg(not(target_arch = "wasm32"))]
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

use custom_engine_core::{
    errors::CoreError, runtime::Runtime, traits::RenderWorker, worker::Worker,
};

use crate::{
    application::AppState,
    config::{EngineConfig, LoadConfig, WorkerKind},
    workers::{
        custom::SimpleCustomRender, model::SimpleModelRender, render_texture::SimpleRenderTexture,
        render_to_texture::SimpleRenderToTexture, simple::SimpleRender,
    },
};

#[derive(Debug)]
pub struct EngineRunner {
    config: EngineConfig,
}

impl EngineRunner {
    pub fn new<C: AsRef<str>>(config: C) -> Result<Self> {
        let config = EngineConfig::load(config)?;

        Ok(Self { config })
    }

    pub fn logger(self) -> Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use std::panic::set_hook;

                set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            } else {
                if let Some(logger) = self.config.logger.as_ref() {
                    log4rs::init_file(logger, Default::default())?;
                } else {
                    let stdout = ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(
                         "- {d(%Y-%m-%d %H:%M:%S)(utc)} {h([{M}])}: {m}{n}",
                    )))
                    .build();
                    let requests = FileAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(
                            "- {d(%Y-%m-%d %H:%M:%S)(utc)} {h([{M}])}: {m}{n}",
                    )))
                    .build("data/engine.log")?;

                    let config = Config::builder()
                        .appender(Appender::builder().build("stdout", Box::new(stdout)))
                        .appender(Appender::builder().build("requests", Box::new(requests)))
                        .logger(
                            Logger::builder()
                                .appender("requests")
                                .additive(true)
                                .build("custom_engine_core", LevelFilter::Info),
                        )
                        .logger(
                            Logger::builder()
                                .appender("requests")
                                .additive(true)
                                .build("wgpu", LevelFilter::Info),
                        )
                        .build(
                            Root::builder()
                                .appender("stdout")
                                .appender("requests")
                                .build(LevelFilter::Info),
                        )?;

                    let _ = log4rs::init_config(config)?;
                }
            }
        }

        Ok(self)
    }

    pub fn run(self) -> Result<()> {
        let (event_loop, window) = self.env_init()?;

        let runtime = Runtime::init(Some(&window))?;
        let mut app_state = AppState::new();
        let mut worker_surface = runtime.worker_surface()?;
        let mut r = SimpleRender::init(&mut worker_surface)?;

        event_loop.run(|event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                r.update(&mut worker_surface, event).unwrap();

                match event {
                    // Worker
                    WindowEvent::RedrawRequested => {
                        match r.render(&mut worker_surface) {
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::Lost)) => {
                                worker_surface.resize()
                            }
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::Timeout)) => {
                                worker_surface.resize()
                            }
                            Err(CoreError::SurfaceError(wgpu::SurfaceError::OutOfMemory)) => {
                                control_flow.exit()
                            }
                            Err(e) => error!("{e}"),
                            _ => {}
                        }

                        window.request_redraw();
                    }
                    WindowEvent::Resized(new_size) => {
                        worker_surface.resize_by_size((new_size.width, new_size.height))
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        worker_surface.resize_by_scale(*scale_factor)
                    }

                    // Mouse
                    WindowEvent::CursorMoved { position, .. } => {
                        app_state.cursor_position = *position;
                        info!("{position:?}");
                    }
                    WindowEvent::MouseInput { state, .. } => {
                        if let ElementState::Pressed = state {
                            app_state.click_state = *state;
                            app_state.click_position = app_state.cursor_position;

                            r.click(&mut worker_surface, &app_state).unwrap();
                        }
                    }

                    // Exit
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => {
                        control_flow.exit();
                    }
                    _ => {}
                }
            }
            _ => {}
        })?;

        Ok(())
    }

    // Helpers
    fn render_init(&self, worker: &mut Worker<'_>) -> Result<Box<dyn RenderWorker>> {
        use WorkerKind::*;

        let rw = match self.config.worker {
            Simple => Box::new(SimpleRender::init(worker)?) as Box<dyn RenderWorker>,
            RenderTexture => Box::new(SimpleRenderTexture::init(worker)?) as Box<dyn RenderWorker>,
            Model => Box::new(SimpleModelRender::init(worker)?) as Box<dyn RenderWorker>,
            Custom => Box::new(SimpleCustomRender::init(worker)?) as Box<dyn RenderWorker>,
            RenderToTexture => {
                Box::new(SimpleRenderToTexture::init(worker)?) as Box<dyn RenderWorker>
            }
        };

        Ok(rw)
    }

    fn env_init(&self) -> Result<(EventLoop<()>, Window)> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height))
            .build(&event_loop)?;

        cfg_if::cfg_if! {
          if #[cfg(target_arch = "wasm32")] {
                use anyhow::anyhow;
                use winit::platform::web::WindowExtWebSys;

                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| {
                        let dst = doc.get_element_by_id("wasm-body")?;
                        let canvas = web_sys::Element::from(window.canvas()?);

                        dst.append_child(&canvas).ok()?;

                        Some(())
                })
                .ok_or(anyhow!("Web Sys window init"))?;

                Ok((event_loop, window))
            } else {
                Ok((event_loop, window))
            }
        }
    }
}
