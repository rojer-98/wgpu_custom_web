use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{errors::CoreError, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct QuerySet {
    pub id: usize,

    #[deref]
    #[deref_mut]
    inner_qs: wgpu::QuerySet,
}

#[derive(Debug)]
pub struct QuerySetBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    query_type: Option<wgpu::QueryType>,
    count: u32,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for QuerySetBuilder<'a> {
    type Final = QuerySet;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            query_type: None,
            id: None,
            label: None,
            count: 1,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            query_type: None,
            id: Some(id),
            label: None,
            count: 1,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let query_set_name = format!("Depth stencil attachment: {id}");

        let label = self.label.unwrap_or(&query_set_name);
        let ty = self
            .query_type
            .ok_or(CoreError::EmptyQueryType(label.to_string()))?;
        let count = self.count;

        debug!(
            "
Build `{label}`:
    Query type: {ty:#?},
    Query count: {count},"
        );

        let inner_qs = self.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some(label),
            count,
            ty,
        });

        Ok(QuerySet { id, inner_qs })
    }
}

impl<'a> QuerySetBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn query_type(mut self, qt: wgpu::QueryType) -> Self {
        self.query_type = Some(qt);
        self
    }
}
