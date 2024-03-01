use std::ops::Deref;

use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{bind_group::layout::BindGroupLayout, errors::CoreError, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct PipelineLayout {
    pub id: usize,

    #[deref]
    #[deref_mut]
    inner_pl: wgpu::PipelineLayout,
}

pub struct PipelineLayoutBuilder<'a> {
    id: Option<usize>,
    entries: Option<Vec<&'a wgpu::BindGroupLayout>>,
    label: Option<&'a str>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for PipelineLayoutBuilder<'a> {
    type Final = PipelineLayout;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            label: None,
            id: None,
            entries: None,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            label: None,
            id: Some(id),
            entries: None,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let layout_name = format!("Pipeline layout: {id}");

        let label = self.label.unwrap_or(&layout_name);
        let entries = self.entries.unwrap_or(vec![]);

        debug!(
            "
Build `{label}`:
    Entries: {entries:#?},"
        );

        let inner_pl = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: entries.as_slice(),
                push_constant_ranges: &[],
            });

        Ok(PipelineLayout { id, inner_pl })
    }
}

impl<'a> PipelineLayoutBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn entry(mut self, bgl: &'a BindGroupLayout) -> Self {
        self.entries.get_or_insert(vec![]).push(bgl.deref());
        self
    }

    pub fn entries(mut self, bgls: Vec<&'a BindGroupLayout>) -> Self {
        self.entries
            .get_or_insert(vec![])
            .extend(bgls.into_iter().map(|bgl| bgl.deref()).collect::<Vec<_>>());
        self
    }
}
