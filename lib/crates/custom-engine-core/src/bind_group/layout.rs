use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{
    bind_group::entry::{BindGroupLayoutEntryBuilder, BindGroupLayoutEntryList},
    errors::CoreError,
    traits::Builder,
};

#[derive(Debug, Deref, DerefMut)]
pub struct BindGroupLayout {
    pub id: usize,

    #[deref]
    #[deref_mut]
    inner_bgl: wgpu::BindGroupLayout,
}

pub struct BindGroupLayoutBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    entries: Option<Vec<wgpu::BindGroupLayoutEntry>>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for BindGroupLayoutBuilder<'a> {
    type Final = BindGroupLayout;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            id: None,
            label: None,
            entries: None,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            label: None,
            entries: None,
            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let layout_name = format!("Bind group layout: {id}");

        let label = self.label.unwrap_or(&layout_name);
        let entries = self
            .entries
            .ok_or(CoreError::EmptyEntries(label.to_string()))?;

        debug!(
            "
Build `{label}`:
    Entries: {entries:#?},"
        );

        let inner_bgl = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: entries.as_slice(),
                label: Some(label),
            });

        Ok(BindGroupLayout { id, inner_bgl })
    }
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn entries(mut self, entry: wgpu::BindGroupLayoutEntry) -> Self {
        self.entries.get_or_insert(vec![]).push(entry);
        self
    }

    pub fn entries_builder(
        mut self,
        entry_builder: BindGroupLayoutEntryBuilder,
    ) -> Result<Self, CoreError> {
        self.entries
            .get_or_insert(vec![])
            .push(*entry_builder.build()?);
        Ok(self)
    }

    pub fn entry_builders_list(
        mut self,
        entries_list: BindGroupLayoutEntryList,
    ) -> Result<Self, CoreError> {
        self.entries
            .get_or_insert(vec![])
            .append(&mut entries_list.into_inner());
        Ok(self)
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}
