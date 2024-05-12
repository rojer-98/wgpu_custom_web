use std::mem::size_of_val;

use image::{ImageBuffer, Rgba};
use log::{debug, info, warn};

use crate::{
    buffer::Buffer,
    errors::CoreError,
    model::Model,
    render_pass::RenderPass,
    runtime::ImageFormat,
    storage::Storages,
    texture::{CopyTextureParams, RenderTexture},
    traits::Builder,
    uniform::Uniforms,
    worker::{View, ViewTexture, Worker},
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

            self.surface_properties.config.width = new_size.0;
            self.surface_properties.config.height = new_size.1;
            self.surface_properties
                .surface
                .configure(&self.device, &self.surface_properties.config);
        }
    }

    #[inline]
    pub fn resize(&mut self) {
        self.resize_by_size(self.size);
    }

    #[inline]
    pub fn render(&self, render_pass: RenderPass<'a>) -> Result<(), CoreError> {
        if let Some(view) = self.view.as_ref() {
            match view {
                View::Texture(ViewTexture {
                    render_texture,
                    buffer,
                    ..
                }) => render_pass
                    .copy_params(CopyTextureParams::new(buffer, render_texture))
                    .render(&self.queue)?,
                View::Surface(_) => render_pass.render(&self.queue)?,
            }
        }

        Ok(())
    }

    #[inline]
    pub fn render_pass(&self) -> RenderPass<'_> {
        RenderPass::new(&self.device, 0)
    }

    // Helpers
    #[inline]
    pub fn load_texture(&self, rt: &RenderTexture) {
        rt.store_to_memory(&self.queue);
    }

    #[inline]
    pub fn load_model(&self, model: &Model) {
        model.load(&self.queue)
    }

    pub fn update_uniform<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let uniform = self.get_uniform_ref(id)?;

        self.update_uniform_direct(&uniform, name, data)
    }

    pub fn update_uniform_direct<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        uniform: &'_ Uniforms,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let buffer = uniform
            .get_buffer(name)
            .ok_or(CoreError::UniformBufferNotFound(name.to_string()))?;

        self.update_buffer_data(buffer, 0, data)
    }

    pub fn update_storage<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let storage = self.get_storage_ref(id)?;

        self.update_storage_direct(&storage, name, data)
    }

    pub fn update_storage_direct<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        storage: &'_ Storages,
        name: &str,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
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

    pub async fn read_uniform<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
    ) -> Result<Vec<T>, CoreError> {
        let uniform = self.get_uniform_ref(id)?;
        let buffer = uniform
            .get_buffer(name)
            .ok_or(CoreError::UniformBufferNotFound(name.to_string()))?;
        let buffer_data = buffer.read_buffer_async(&self.device).await?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub async fn read_storage_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
        name: &str,
    ) -> Result<Vec<T>, CoreError> {
        let storage = self.get_storage_ref(id)?;
        let buffer = storage
            .get_buffer(name)
            .ok_or(CoreError::StorageNotFound(name.to_string()))?;
        let buffer_data = buffer.read_buffer_async(&self.device).await?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub async fn read_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: usize,
    ) -> Result<Vec<T>, CoreError> {
        let buffer = self.get_buffer_ref(id)?;
        let buffer_data = buffer.read_buffer_async(&self.device).await?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub async fn read_plain_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        buffer: &'_ Buffer,
    ) -> Result<Vec<T>, CoreError> {
        let buffer_data = buffer.read_buffer_async(&self.device).await?;

        let cast_data: &[T] = bytemuck::cast_slice(&buffer_data);

        Ok(cast_data.to_vec())
    }

    pub fn view_texture(
        &mut self,
        image_format: ImageFormat,
        path_to_save: String,
    ) -> Result<wgpu::TextureView, CoreError> {
        let (render_texture, buffer) = self.init_runtime_texture()?;
        self.view = Some(View::Texture(ViewTexture {
            render_texture,
            image_format,
            path_to_save,
            buffer,
        }));

        self.view()
    }

    pub fn view_surface(&mut self) -> Result<wgpu::TextureView, CoreError> {
        self.view = Some(View::Surface(
            self.surface_properties.surface.get_current_texture()?,
        ));

        self.view()
    }

    pub async fn present(&mut self) -> Result<(), CoreError> {
        let v = self.view.take();

        match v {
            Some(View::Surface(s)) => s.present(),
            Some(View::Texture(t)) => {
                let data = t.buffer.read_buffer_async(&self.device).await?;
                let i_b = ImageBuffer::<Rgba<u8>, _>::from_raw(self.size.0, self.size.1, data)
                    .ok_or(CoreError::ImageBufferCreate)?;
                let save_path = format!("{}.{}", t.path_to_save.clone(), t.image_format);

                debug!("Save texture to {save_path}");
                i_b.save(save_path)?;

                //self.view = Some(View::Texture(t));
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
    //pub(crate) fn init_with_size(&mut self, size: (u32, u32)) -> Result<(), CoreError> {
    //    self.size = size;

    //    self.init()
    //}

    // Private helpers
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
            .create_buffer()
            .label("Render texture buffer")
            .binding(0)
            .size((self.size.0 * self.size.1 * components).into())
            .usage(wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ)
            .build()?;

        self.format = format;

        Ok((t, b))
    }

    #[inline(always)]
    fn view(&mut self) -> Result<wgpu::TextureView, CoreError> {
        Ok(self
            .view
            .as_ref()
            .ok_or(CoreError::NotInitView)?
            .texture_view())
    }

    fn update_buffer_data<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        b: &Buffer,
        offset: u64,
        data: &'_ [T],
    ) -> Result<(), CoreError> {
        let buffer_size = b.size();
        let data_len = (size_of_val(data) as u64) + offset;

        if data_len > buffer_size {
            return Err(CoreError::WrongBufferSize);
        } else {
            self.queue
                .write_buffer(b, offset, bytemuck::cast_slice(data));
        }

        Ok(())
    }
}
