use derive_more::{Deref, DerefMut};
use log::debug;

use crate::errors::CoreError;

#[derive(Debug, Deref, DerefMut)]
pub struct BindGroupLayoutEntryList(Vec<BindGroupLayoutEntry>);

impl BindGroupLayoutEntryList {
    pub fn into_inner(self) -> Vec<wgpu::BindGroupLayoutEntry> {
        self.0.into_iter().map(|e| e.inner_bgle).collect()
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct BindGroupLayoutEntry {
    pub id: usize,
    pub binding: u32,

    #[deref]
    #[deref_mut]
    inner_bgle: wgpu::BindGroupLayoutEntry,
}

pub struct BindGroupLayoutEntryBuilder {
    id: Option<usize>,
    binding: Option<u32>,
    visibility: wgpu::ShaderStages,
    ty: Option<wgpu::BindingType>,
}

impl<'a> BindGroupLayoutEntryBuilder {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            id: None,
            binding: None,
            ty: None,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }

    pub fn new_indexed(id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            binding: None,
            ty: None,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }

    pub fn build(self) -> Result<BindGroupLayoutEntry, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let entry_name = format!("Bind layout entry: {id}");

        let visibility = self.visibility;
        let binding = self
            .binding
            .ok_or(CoreError::EmptyBinding(entry_name.clone()))?;
        let ty = self
            .ty
            .ok_or(CoreError::EmptyBindType(entry_name.clone()))?;

        debug!(
            "
Build `{entry_name}`:
    Binding: {binding},
    Visibility: {visibility:#?},
    Ty: {ty:#?},"
        );

        let inner_bgle = wgpu::BindGroupLayoutEntry {
            binding,
            ty,
            visibility,
            count: None,
        };

        Ok(BindGroupLayoutEntry {
            id,
            inner_bgle,
            binding,
        })
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = Some(binding);
        self
    }

    pub fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn ty(mut self, ty: wgpu::BindingType) -> Self {
        self.ty = Some(ty);
        self
    }
}
