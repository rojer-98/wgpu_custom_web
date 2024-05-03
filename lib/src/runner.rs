use anyhow::Result;
use instant::Duration;
use log::error;
#[cfg(not(target_arch = "wasm32"))]
use log::LevelFilter;
#[cfg(not(target_arch = "wasm32"))]
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use pollster::block_on;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder},
    keyboard::{Key, NamedKey},
    window::Window,
};

use custom_engine_core::{errors::CoreError, runtime::Runtime, traits::RenderWorker};

use crate::{
    application::foreign::UserEvent,
    config::{EngineConfig, LoadConfig},
    workers::model::SimpleModelRender,
};

#[cfg(target_arch = "wasm32")]
pub static mut EVENT_LOOP_PROXY: Option<EventLoopProxy<UserEvent>> = None;

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

    pub async fn run(self) -> Result<()> {
        let (event_loop, window) = self.env_init()?;

        let runtime = Runtime::init(Some(&window)).await?;
        //let mut app_state = AppState::new();
        let mut worker_surface = runtime.worker_surface()?;
        let mut r = SimpleModelRender::init(&mut worker_surface).await?;

        event_loop.run(|event, control_flow| match event {
            Event::UserEvent(u_e) => u_e.on_event(),

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                r.update(&mut worker_surface, event, Duration::from_secs(1))
                    .unwrap();

                match event {
                    // Worker
                    WindowEvent::RedrawRequested => {
                        match block_on(async { r.render(&mut worker_surface).await }) {
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
                        worker_surface.resize_by_size((new_size.width, new_size.height));
                        r.resize(&mut worker_surface).unwrap();
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        worker_surface.resize_by_scale(*scale_factor);
                        r.resize(&mut worker_surface).unwrap();
                    }

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

    fn env_init(&self) -> Result<(EventLoop<UserEvent>, Window)> {
        let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(self.config.width, self.config.height)),
        )?;

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

                unsafe {
                    EVENT_LOOP_PROXY = Some(event_loop.create_proxy());
                }

                Ok((event_loop, window))
            } else {
                Ok((event_loop, window))
            }
        }
    }
}
