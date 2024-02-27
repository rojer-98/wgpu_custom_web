use std::{env, fmt::Debug, str::FromStr};

use anyhow::{anyhow, Result};
use log::trace;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Deserialize, Default, Clone, Copy)]
pub enum WorkerKind {
    #[default]
    Simple,
    Custom,
    Model,
    RenderTexture,
    RenderToTexture,
}

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub logger: Option<String>,
    #[serde(default)]
    pub worker: WorkerKind,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

pub trait LoadConfig {
    fn load<C: AsRef<str>>(config: C) -> Result<Self>
    where
        Self: Sized + DeserializeOwned + Debug;
}

impl<T: Sized + DeserializeOwned + Debug> LoadConfig for T {
    fn load<C: AsRef<str>>(config: C) -> Result<Self>
    where
        Self: Sized + DeserializeOwned + Debug,
    {
        let mut params = serde_yaml::from_str(config.as_ref())?;

        expand_variables(&mut params);

        let config = serde_yaml::to_string(&params)?;
        let params: Result<T, serde_yaml::Error> = serde_yaml::from_str(&config);

        if let Ok("1") = env::var("DEBUG_CONFIG").as_deref() {
            trace!("Full processed config:\n{config}");
        }

        if let Err(e) = &params {
            if let Some(location) = e.location() {
                let start = location.line().saturating_sub(5);
                let end = location.line() + 5;
                let mut msg = format!(
                    "{e}\nRelevant part of the config (set DEBUG_CONFIG=1 to print full config):\n",
                );

                for (index, line) in config.lines().enumerate().skip(start).take(end - start) {
                    let tag0 = if index + 1 == location.line() {
                        "\x1b[31;1m"
                    } else {
                        ""
                    };

                    let tag1 = if index + 1 == location.line() {
                        "\x1b[0m"
                    } else {
                        ""
                    };

                    let inc = index + 1;
                    msg += format!("{tag0}{inc:>3}: {line}{tag1}\n").as_str();
                }

                return Err(anyhow!("{msg}"));
            } else {
                return Err(anyhow!("{e} (set DEBUG_CONFIG=1 to print full config)"));
            }
        }

        Ok(params?)
    }
}

/// This function is used for scan every config's string parameter and replace environment variables inside
///
/// # String examples with replacement
///
/// * `/mypath/${ENV_VAR_NAME}/bla/bla`
/// * `My name is ${APP_NAME}. I have version ${APP_VERSION}`
///
/// # String examples without replacement
///
/// * `/mypath/\${NOT_ENV_VAR_NAME}/bla/bla`
/// * `My name is \${WHAT_IS_MY_NAME}`
///
/// Be aware: in `yml` files you must use `\\` for a single backslash. So every backslash in these examples actually must be doubled.
fn subst_env_variable(value: &str) -> String {
    let mut acc = String::with_capacity(value.len());
    let mut split = value.split("${");

    // split always has at least a single value
    acc.push_str(split.next().unwrap_or_default());

    split.for_each(|part| {
        // check if `${` was prefixed with escaping slash `\`
        if acc.ends_with("\\\\") {
            // if `${` was prefixed by double escaping char
            // then it is escaping char for escaping char => we must remove last one
            acc.pop();
        } else if acc.ends_with('\\') {
            // if it was prefixed by `\`, then delete that escaping character
            acc.pop();

            // and skip all the logic of env variable replacement
            acc.push_str("${");
            acc.push_str(part);
            return;
        }

        if let Some((varname, tail)) = part.split_once('}') {
            // trim ":" prefix
            let varname = varname.split_once(':');

            if let Some((value, content)) = varname {
                match env::var(value) {
                    Ok(v) => {
                        acc.push_str(&v);
                    }
                    Err(_) => acc.push_str(content),
                }
            }

            acc.push_str(tail);
        } else {
            // if no closing bracket were found, then just appending raw content
            acc.push_str("${");
            acc.push_str(part);
        }
    });

    acc
}

fn expand_variables(value: &mut serde_yaml::Value) {
    use serde_yaml::*;

    match value {
        Value::String(text) => {
            let v = subst_env_variable(text.as_str());

            if v == *text {
                return;
            }

            if let Ok(v) = u64::from_str(&v) {
                *value = Value::Number(v.into());
                return;
            }

            if let Ok(v) = f64::from_str(&v) {
                *value = Value::Number(v.into());
                return;
            }

            if let Ok(v) = bool::from_str(&v) {
                *value = Value::Bool(v);
                return;
            }

            *text = v;
        }
        Value::Mapping(mapping) => {
            for (_, v) in mapping {
                expand_variables(v);
            }
        }
        Value::Sequence(seq) => {
            for v in seq {
                expand_variables(v);
            }
        }
        _ => {}
    }
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    600
}
