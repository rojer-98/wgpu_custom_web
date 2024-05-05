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
    application::{foreign::UserEvent, AppState},
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
        EventLoop::<UserEvent>::with_user_event()
            .build()?
            .run_app(&mut Runtime::new((1600, 1200)).add_render::<SimpleModelRender>())?;

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
