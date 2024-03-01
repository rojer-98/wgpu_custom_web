use std::mem::size_of;

use image::{ImageBuffer, Rgba};
use log::{debug, error, info, warn};
use pollster::block_on;

use crate::{
    buffer::Buffer,
    errors::CoreError,
    model::Model,
    render_pass::RenderPass,
    runtime::RuntimeKind,
    texture::{CopyTextureParams, RenderTexture},
    traits::Builder,
    worker::{View, Worker},
};

impl<'a> Worker<'a> {
    #[inline]
    pub fn resize_by_scale(&mut self, new_scale_factor: f64) {
        if self.scale_factor > 0. {
            let (w, h) = (
                ((self.size.0 as f64 / self.scale_factor) * new_scale_factor) as u32,
                ((self.size.1 as f64 / self.scale_factor) * new_scale_factor) as u32,
            );
            let (a_w, a_h) = (
                self.limits.max_texture_dimension_2d,
                self.limits.max_texture_dimension_2d,
            );

            self.size.0 = if w > a_w {
                warn!("New `width` {w} is more than maximum. Set the max `width`: {a_w}");
                a_w
            } else {
                w
            };
            self.size.1 = if h > a_h {
                warn!("New `height` {h} is more than maximum. Set the max `height`: {a_h}");
                a_h
            } else {
                h
            };
            info!("Resize with size: {:?}", self.size);
            self.scale_factor = new_scale_factor;
            self.resize();
        }
    }

    #[inline]
    pub fn resize_by_size(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;

            match &mut self.runtime_kind {
                RuntimeKind::Winit(s_p) => {
                    s_p.config.width = new_size.0;
                    s_p.config.height = new_size.1;
                    s_p.surface.configure(&self.device, &s_p.config);
                }
                RuntimeKind::Texture(_, _) => {
                    if let Err(e) = self.init_runtime_texture() {
                        panic!("{e}");
                    }
                }
            }
        }
    }

    #[inline]
    pub fn resize(&mut self) {
        self.resize_by_size(self.size);
    }

    #[inline]
    pub fn render(&self, render_pass: RenderPass<'_>) -> Result<(), CoreError> {
        if let Some(View::Texture(t, b)) = self.view.as_ref() {
            render_pass
                .copy_params(CopyTextureParams::new(&b, &t))
                .render(&self.queue)
        } else {
            render_pass.render(&self.queue)
        }
    }

    #[inline]
    pub fn render_pass(&self) -> RenderPass<'_> {
        RenderPass::new(&self.device, 0)
    }

    // Helpers
    #[inline]
    pub fn load_texture(&self, rt: &RenderTexture) {
        rt.store_to_memory(self.queue);
    }

    #[inline]
    pub fn load_model(&self, model: &Model) {
        model.load(self.queue)
    }

    pub fn update_uniform<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let uniform = self.get_uniform_ref(id)?;
        let buffer = uniform
            .get_buffer(name)
            .ok_or(CoreError::UniformBufferNotFound(name.to_string()))?;

        self.update_buffer_data(buffer, 0, data)
    }

    pub fn update_storage_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let storage = self.get_storage_ref(id)?;
        let buffer = storage
            .get_buffer(name)
            .ok_or(CoreError::StorageNotFound(name.to_string()))?;

        self.update_buffer_data(buffer, 0, data)
    }

    pub fn update_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        offset: u64,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let buffer = self.get_buffer_ref(id)?;

        self.update_buffer_data(&buffer, offset, data)
    }

    pub fn update_buffer_direct<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        buffer: &'_ Buffer,
        offset: u64,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        self.update_buffer_data(buffer, offset, data)
    }

    pub fn read_uniform<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
    ) -> Result<Vec<T>, CoreError> {
        let uniform = self.get_uniform_ref(id)?;
        let buffer = uniform
            .get_buffer(name)
            .ok_or(CoreError::UniformBufferNotFound(name.to_string()))?;
        let buffer_data = block_on(async { buffer.read_buffer_async(self.device).await })?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub fn read_storage_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
    ) -> Result<Vec<T>, CoreError> {
        let storage = self.get_storage_ref(id)?;
        let buffer = storage
            .get_buffer(name)
            .ok_or(CoreError::StorageNotFound(name.to_string()))?;
        let buffer_data = block_on(async { buffer.read_buffer_async(self.device).await })?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub fn read_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
    ) -> Result<Vec<T>, CoreError> {
        let buffer = self.get_buffer_ref(id)?;
        let buffer_data = block_on(async { buffer.read_buffer_async(self.device).await })?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub fn read_plain_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        buffer: &'_ Buffer,
    ) -> Result<Vec<T>, CoreError> {
        let buffer_data = block_on(async { buffer.read_buffer_async(self.device).await })?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub fn texture_view(&mut self) -> Result<wgpu::TextureView, CoreError> {
        Ok(self.view()?.texture_view())
    }

    pub fn view(&mut self) -> Result<&View, CoreError> {
        if self.view.is_none() {
            self.init()?;
        }

        self.view.as_ref().ok_or(CoreError::NotInitView)
    }

    pub fn present(&mut self) -> Result<(), CoreError> {
        let v = self.view.take();

        match v {
            Some(View::Surface(s)) => s.present(),
            Some(View::Texture(rt, b)) => {
                if let RuntimeKind::Texture(path_to, kind) = &self.runtime_kind {
                    let data = block_on(async { b.read_buffer_async(&self.device).await })?;
                    let i_b = ImageBuffer::<Rgba<u8>, _>::from_raw(self.size.0, self.size.1, data)
                        .ok_or(CoreError::ImageBufferCreate)?;
                    let save_path = format!("{path_to}.{kind}");

                    debug!("Save texture to {save_path}");
                    i_b.save(save_path)?;
                }

                self.view = Some(View::Texture(rt, b));
            }
            _ => {}
        }

        Ok(())
    }

    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    // Protected helpers
    pub(crate) fn init_with_size(&mut self, size: (u32, u32)) -> Result<(), CoreError> {
        self.size = size;

        self.init()
    }

    // Private helpers
    fn init(&mut self) -> Result<(), CoreError> {
        self.view = Some(match &self.runtime_kind {
            RuntimeKind::Winit(s_p) => View::Surface(s_p.surface.get_current_texture()?),
            RuntimeKind::Texture(_, _) => {
                let (t, b) = self.init_runtime_texture()?;

                View::Texture(t, b)
            }
        });

        Ok(())
    }

    fn init_runtime_texture(&mut self) -> Result<(RenderTexture, Buffer), CoreError> {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let aspect = wgpu::TextureAspect::All;
        let components = format.components_with_aspect(aspect) as u32;

        let t = self
            .create_render_texture()
            .is_sampler(false)
            .texture_desc(wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.size.0,
                    height: self.size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Render Texture"),
                view_formats: &[format],
            })
            .build()?;
        let b = self
            .create_buffer::<()>()
            .label("Render texture buffer")
            .binding(0)
            .size((self.size.0 * self.size.1 * components).into())
            .usage(wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ)
            .build()?;

        self.format = format;

        Ok((t, b))
    }

    fn update_buffer_data<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        b: &Buffer,
        offset: u64,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let buffer_size = b.size();
        let data_len = ((data.len() * size_of::<T>()) as u64) + offset;

        if data_len > buffer_size {
            return Err(CoreError::WrongBufferSize);
        } else {
            self.queue
                .write_buffer(&b, offset, bytemuck::cast_slice(data));
        }

        Ok(())
    }
}
