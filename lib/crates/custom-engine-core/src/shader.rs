use std::borrow::Cow;

use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{errors::CoreError, traits::Builder};

#[derive(Debug)]
pub enum ShaderKind {
    Fragment(Vec<Option<wgpu::ColorTargetState>>),
    Vertex(Vec<wgpu::VertexBufferLayout<'static>>),
}

#[derive(Debug)]
pub enum ShaderSource<'a> {
    Plain(wgpu::ShaderSource<'a>),
    SPIRV(Vec<u32>),
}

#[derive(Debug, Deref, DerefMut)]
pub struct Shader {
    pub id: usize,
    pub fs_entry_point: String,
    pub fs_options: Vec<Option<wgpu::ColorTargetState>>,
    pub vs_entry_point: String,
    pub vs_options: Vec<wgpu::VertexBufferLayout<'static>>,
    pub compute_entry_point: Option<String>,

    #[deref]
    #[deref_mut]
    inner_shader: wgpu::ShaderModule,
}

impl Shader {
    pub fn make_vertex_state(&self) -> wgpu::VertexState {
        wgpu::VertexState {
            module: &self.inner_shader,
            entry_point: &self.vs_entry_point,
            buffers: &self.vs_options,
        }
    }

    pub fn make_fragment_state(&self) -> wgpu::FragmentState {
        wgpu::FragmentState {
            module: &self.inner_shader,
            entry_point: &self.fs_entry_point,
            targets: &self.fs_options,
        }
    }
}

#[derive(Debug)]
pub struct ShaderBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    fs_entry_point: Option<&'a str>,
    fs_options: Option<Vec<wgpu::ColorTargetState>>,
    vs_entry_point: Option<&'a str>,
    vs_options: Option<Vec<wgpu::VertexBufferLayout<'static>>>,
    source: Option<ShaderSource<'a>>,
    compute_entry_point: Option<&'a str>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for ShaderBuilder<'a> {
    type Final = Shader;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            id: None,
            label: None,
            source: None,
            fs_entry_point: None,
            fs_options: None,
            vs_entry_point: None,
            vs_options: None,
            compute_entry_point: None,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            id: Some(id),
            label: None,
            source: None,
            fs_entry_point: None,
            fs_options: None,
            vs_entry_point: None,
            vs_options: None,
            compute_entry_point: None,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let shader_name = format!("Shader: {id}");

        let label = self.label.unwrap_or(&shader_name);
        let fs_entry_point = self
            .fs_entry_point
            .ok_or(CoreError::EmptyEntryPoint(label.to_string()))?
            .to_string();
        let vs_entry_point = self
            .vs_entry_point
            .ok_or(CoreError::EmptyEntryPoint(label.to_string()))?
            .to_string();
        let vs_options = self.vs_options.unwrap_or(vec![]);
        let fs_options = self
            .fs_options
            .map(|options| options.into_iter().map(Some).collect())
            .unwrap_or(vec![]);
        let compute_entry_point = self.compute_entry_point.map(String::from);

        let source = self
            .source
            .ok_or(CoreError::EmptyShaderSource(label.to_string()))?;

        debug!(
            "
Build `{label}`: 
    Vertex entry point: {vs_entry_point},
    Fragment entry point: {fs_entry_point},
    Vertex options: {vs_options:#?},
    Fragment options: {fs_options:#?},
    Source: {source:#?},
    "
        );

        let inner_shader = match source {
            ShaderSource::Plain(source) => {
                self.device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(label),
                        source,
                    })
            }
            ShaderSource::SPIRV(source) => unsafe {
                self.device
                    .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                        label: Some(label),
                        source: Cow::Borrowed(&source),
                    })
            },
        };

        Ok(Shader {
            id,
            fs_entry_point,
            fs_options,
            vs_entry_point,
            vs_options,
            inner_shader,
            compute_entry_point,
        })
    }
}

impl<'a> ShaderBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn compute_entry_point(mut self, compute_entry_point: &'a str) -> Self {
        self.compute_entry_point = Some(compute_entry_point);
        self
    }

    pub fn fs_entry_point(mut self, entry_point: &'a str) -> Self {
        self.fs_entry_point = Some(entry_point);
        self
    }

    pub fn vs_entry_point(mut self, entry_point: &'a str) -> Self {
        self.vs_entry_point = Some(entry_point);
        self
    }

    pub fn source(mut self, source: wgpu::ShaderSource<'a>) -> Self {
        self.source = Some(ShaderSource::Plain(source));
        self
    }

    pub fn source_data(mut self, source: Vec<u32>) -> Self {
        self.source = Some(ShaderSource::SPIRV(source));
        self
    }

    pub fn fs_options(mut self, options: Vec<wgpu::ColorTargetState>) -> Self {
        self.fs_options = Some(options);
        self
    }

    pub fn vs_options(mut self, options: Vec<wgpu::VertexBufferLayout<'static>>) -> Self {
        self.vs_options = Some(options);
        self
    }
}
