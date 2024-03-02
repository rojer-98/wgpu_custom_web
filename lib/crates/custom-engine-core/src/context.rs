use std::collections::{HashMap, HashSet};

use log::warn;
use rand::{thread_rng, Rng};

use crate::{
    bind_group::{layout::BindGroupLayout, BindGroup},
    buffer::Buffer,
    errors::CoreError,
    model::Model,
    pipeline::{layout::PipelineLayout, Pipeline},
    shader::Shader,
    storage::Storages,
    texture::{DepthTexture, RenderTexture},
    uniform::Uniforms,
    utils::Ref,
};

#[derive(Debug, Default)]
pub struct Context {
    buffers: HashMap<usize, Buffer>,
    bind_groups: HashMap<usize, BindGroup>,
    bind_group_layouts: HashMap<usize, BindGroupLayout>,
    pipelines: HashMap<usize, Pipeline>,
    pipeline_layouts: HashMap<usize, PipelineLayout>,
    shaders: HashMap<usize, Shader>,
    render_textures: HashMap<usize, RenderTexture>,
    depth_textures: HashMap<usize, DepthTexture>,
    process_textures: HashMap<usize, RenderTexture>,
    models: HashMap<usize, Model>,
    uniforms: HashMap<usize, Uniforms>,
    storages: HashMap<usize, Storages>,

    ids: HashSet<usize>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn generate_unique_id(&self) -> usize {
        let mut rng = thread_rng();
        let mut val = rng.gen();

        while self.ids.contains(&val) {
            val = rng.gen();
        }

        val
    }

    // Storages
    #[inline]
    pub fn add_storage(&mut self, u: Storages) {
        if self.storages.contains_key(&u.id) {
            warn!("Storages with id: {} exist in `context`", u.id);
        } else {
            let _ = self.ids.insert(u.id);
            let _ = self.storages.insert(u.id, u);
        }
    }

    #[inline]
    pub fn replace_storage(&mut self, id: usize, mut u: Storages) -> Result<(), CoreError> {
        if self.storages.contains_key(&id) {
            u.id = id;
            *(self.get_storage_mut(id)?) = u;
        } else {
            warn!("Storages with id: {id} doesn't exist in `context`");
        }

        Ok(())
    }

    #[inline]
    pub fn get_storage(&self, id: usize) -> Result<&Storages, CoreError> {
        self.storages
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Storages".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_storage_mut(&mut self, id: usize) -> Result<&mut Storages, CoreError> {
        self.storages
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Storages".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_storage_ref(&self, id: usize) -> Result<Ref<Storages>, CoreError> {
        let val = self
            .storages
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Storages".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_storage(&mut self, id: usize) -> Result<Storages, CoreError> {
        self.storages
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Storages".to_string(),
                id,
            ))
    }

    // Uniforms
    #[inline]
    pub fn add_uniform(&mut self, u: Uniforms) {
        if self.uniforms.contains_key(&u.id) {
            warn!("Uniforms with id: {} exist in `context`", u.id);
        } else {
            let _ = self.ids.insert(u.id);
            let _ = self.uniforms.insert(u.id, u);
        }
    }

    #[inline]
    pub fn replace_uniform(&mut self, id: usize, mut u: Uniforms) -> Result<(), CoreError> {
        if self.uniforms.contains_key(&id) {
            u.id = id;
            *(self.get_uniform_mut(id)?) = u;
        } else {
            warn!("Uniforms with id: {id} doesn't exist in `context`");
        }

        Ok(())
    }

    #[inline]
    pub fn get_uniform(&self, id: usize) -> Result<&Uniforms, CoreError> {
        self.uniforms
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Uniforms".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_uniform_mut(&mut self, id: usize) -> Result<&mut Uniforms, CoreError> {
        self.uniforms
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Uniforms".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_uniform_ref(&self, id: usize) -> Result<Ref<Uniforms>, CoreError> {
        let val = self
            .uniforms
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Uniforms".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_uniform(&mut self, id: usize) -> Result<Uniforms, CoreError> {
        self.uniforms
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Uniforms".to_string(),
                id,
            ))
    }

    // Model
    #[inline]
    pub fn add_model(&mut self, m: Model) {
        if self.models.contains_key(&m.id) {
            warn!("Model with id: {} exist in `context`", m.id);
        } else {
            let _ = self.ids.insert(m.id);
            let _ = self.models.insert(m.id, m);
        }
    }

    #[inline]
    pub fn replace_model(&mut self, id: usize, mut m: Model) -> Result<(), CoreError> {
        if self.models.contains_key(&id) {
            m.id = id;
            *(self.get_model_mut(id)?) = m;
        } else {
            warn!("Model with id: {id} doesn't exist in `context`");
        }

        Ok(())
    }

    #[inline]
    pub fn get_model(&self, id: usize) -> Result<&Model, CoreError> {
        self.models
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Model".to_string(), id))
    }

    #[inline]
    pub fn get_model_mut(&mut self, id: usize) -> Result<&mut Model, CoreError> {
        self.models
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Model".to_string(), id))
    }

    #[inline]
    pub fn get_model_ref(&self, id: usize) -> Result<Ref<Model>, CoreError> {
        let val = self
            .models
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Model".to_string(), id))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_model(&mut self, id: usize) -> Result<Model, CoreError> {
        self.models
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Model".to_string(), id))
    }

    // Buffer
    #[inline]
    pub fn add_buffer(&mut self, b: Buffer) {
        if self.buffers.contains_key(&b.id) {
            warn!("Buffer with id: {} exist in `context`", b.id);
        } else {
            let _ = self.ids.insert(b.id);
            let _ = self.buffers.insert(b.id, b);
        }
    }

    #[inline]
    pub fn replace_buffer(&mut self, id: usize, mut b: Buffer) -> Result<(), CoreError> {
        if self.buffers.contains_key(&id) {
            b.id = id;
            *(self.get_buffer_mut(id)?) = b;
        } else {
            warn!("Buffer with id: {id} doesn't exist in `context`");
        }

        Ok(())
    }

    #[inline]
    pub fn get_buffer(&self, id: usize) -> Result<&Buffer, CoreError> {
        self.buffers
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Buffer".to_string(), id))
    }

    #[inline]
    pub fn get_buffer_mut(&mut self, id: usize) -> Result<&mut Buffer, CoreError> {
        self.buffers
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Buffer".to_string(), id))
    }

    #[inline]
    pub fn get_buffer_ref(&self, id: usize) -> Result<Ref<Buffer>, CoreError> {
        let val = self
            .buffers
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Buffer".to_string(), id))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_buffer(&mut self, id: usize) -> Result<Buffer, CoreError> {
        self.buffers
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Buffer".to_string(), id))
    }

    // Bind group
    #[inline]
    pub fn add_bind_group_layout(&mut self, bgl: BindGroupLayout) {
        if self.bind_group_layouts.contains_key(&bgl.id) {
            warn!("BindGroupLayout with id: {} exist in `context`", bgl.id);
        } else {
            let _ = self.ids.insert(bgl.id);
            let _ = self.bind_group_layouts.insert(bgl.id, bgl);
        }
    }

    #[inline]
    pub fn replace_bind_group_layout(
        &mut self,
        id: usize,
        mut bgl: BindGroupLayout,
    ) -> Result<(), CoreError> {
        if self.bind_group_layouts.contains_key(&id) {
            bgl.id = id;
            *(self.get_bind_group_layout_mut(id)?) = bgl;
        } else {
            warn!("BindGroupLayout with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_bind_group_layout(&self, id: usize) -> Result<&BindGroupLayout, CoreError> {
        self.bind_group_layouts
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroupLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_bind_group_layout_mut(
        &mut self,
        id: usize,
    ) -> Result<&mut BindGroupLayout, CoreError> {
        self.bind_group_layouts
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroupLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_bind_group_layout_ref(&self, id: usize) -> Result<Ref<BindGroupLayout>, CoreError> {
        let val = self
            .bind_group_layouts
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroupLayout".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_bind_group_layout(&mut self, id: usize) -> Result<BindGroupLayout, CoreError> {
        self.bind_group_layouts
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroupLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn add_bind_group(&mut self, bg: BindGroup) {
        if self.bind_groups.contains_key(&bg.id) {
            warn!("BindGroup with id: {} exist in `context`", bg.id);
        } else {
            let _ = self.ids.insert(bg.id);
            let _ = self.bind_groups.insert(bg.id, bg);
        }
    }

    #[inline]
    pub fn replace_bind_group(&mut self, id: usize, mut bg: BindGroup) -> Result<(), CoreError> {
        if self.bind_groups.contains_key(&id) {
            bg.id = id;
            *(self.get_bind_group_mut(id)?) = bg;
        } else {
            warn!("BindGroup with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_bind_group(&self, id: usize) -> Result<&BindGroup, CoreError> {
        self.bind_groups
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroup".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_bind_group_mut(&mut self, id: usize) -> Result<&mut BindGroup, CoreError> {
        self.bind_groups
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroup".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_bind_group_ref(&self, id: usize) -> Result<Ref<BindGroup>, CoreError> {
        let val = self
            .bind_groups
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroup".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_bind_group(&mut self, id: usize) -> Result<BindGroup, CoreError> {
        self.bind_groups
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "BindGroup".to_string(),
                id,
            ))
    }

    // Pipeline
    #[inline]
    pub fn add_pipeline_layout(&mut self, pl: PipelineLayout) {
        if self.pipeline_layouts.contains_key(&pl.id) {
            warn!("PipelineLayout with id: {} exist in `context`", pl.id);
        } else {
            let _ = self.ids.insert(pl.id);
            let _ = self.pipeline_layouts.insert(pl.id, pl);
        }
    }

    #[inline]
    pub fn replace_pipeline_layout(
        &mut self,
        id: usize,
        mut pl: PipelineLayout,
    ) -> Result<(), CoreError> {
        if self.pipeline_layouts.contains_key(&id) {
            pl.id = id;
            *(self.get_pipeline_layout_mut(id)?) = pl;
        } else {
            warn!("PipelineLayout with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_pipeline_layout(&self, id: usize) -> Result<&PipelineLayout, CoreError> {
        self.pipeline_layouts
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "PipelineLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_pipeline_layout_mut(&mut self, id: usize) -> Result<&mut PipelineLayout, CoreError> {
        self.pipeline_layouts
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "PipelineLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_pipeline_layout_ref(&self, id: usize) -> Result<Ref<PipelineLayout>, CoreError> {
        let val = self
            .pipeline_layouts
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "PipelineLayout".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_pipeline_layout(&mut self, id: usize) -> Result<PipelineLayout, CoreError> {
        self.pipeline_layouts
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "PipelineLayout".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn add_pipeline(&mut self, p: Pipeline) {
        if self.pipelines.contains_key(&p.id) {
            warn!("Pipeline with id: {} exist in `context`", p.id);
        } else {
            let _ = self.ids.insert(p.id);
            let _ = self.pipelines.insert(p.id, p);
        }
    }

    #[inline]
    pub fn replace_pipeline(&mut self, id: usize, mut p: Pipeline) -> Result<(), CoreError> {
        if self.pipelines.contains_key(&id) {
            p.id = id;
            *(self.get_pipeline_mut(id)?) = p;
        } else {
            warn!("Pipeline with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_pipeline(&self, id: usize) -> Result<&Pipeline, CoreError> {
        self.pipelines
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Pipeline".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_pipeline_mut(&mut self, id: usize) -> Result<&mut Pipeline, CoreError> {
        self.pipelines
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Pipeline".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_pipeline_ref(&self, id: usize) -> Result<Ref<Pipeline>, CoreError> {
        let val = self
            .pipelines
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Pipeline".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_pipeline(&mut self, id: usize) -> Result<Pipeline, CoreError> {
        self.pipelines
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Pipeline".to_string(),
                id,
            ))
    }

    // Shader
    #[inline]
    pub fn add_shader(&mut self, sh: Shader) {
        if self.shaders.contains_key(&sh.id()) {
            warn!("Shader with id: {} exist in `context`", sh.id());
        } else {
            let _ = self.ids.insert(sh.id());
            let _ = self.shaders.insert(sh.id(), sh);
        }
    }

    #[inline]
    pub fn replace_shader(&mut self, id: usize, mut sh: Shader) -> Result<(), CoreError> {
        if self.shaders.contains_key(&id) {
            *(sh.id_mut()) = id;
            *(self.get_shader_mut(id)?) = sh;
        } else {
            warn!("Shader with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_shader(&self, id: usize) -> Result<&Shader, CoreError> {
        self.shaders
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Shader".to_string(), id))
    }

    #[inline]
    pub fn get_shader_mut(&mut self, id: usize) -> Result<&mut Shader, CoreError> {
        self.shaders
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Shader".to_string(), id))
    }

    #[inline]
    pub fn get_shader_ref(&self, id: usize) -> Result<Ref<Shader>, CoreError> {
        let val = self
            .shaders
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Shader".to_string(), id))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_shader(&mut self, id: usize) -> Result<Shader, CoreError> {
        self.shaders
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist("Shader".to_string(), id))
    }

    // Render Texture
    #[inline]
    pub fn add_render_texture(&mut self, rt: RenderTexture) {
        if self.render_textures.contains_key(&rt.id) {
            warn!("Render Texture with id: {} exist in `context`", rt.id);
        } else {
            let _ = self.ids.insert(rt.id);
            let _ = self.render_textures.insert(rt.id, rt);
        }
    }

    #[inline]
    pub fn replace_render_texture(
        &mut self,
        id: usize,
        mut rt: RenderTexture,
    ) -> Result<(), CoreError> {
        if self.render_textures.contains_key(&id) {
            rt.id = id;
            *(self.get_render_texture_mut(id)?) = rt;
        } else {
            warn!("Render Texture with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_render_texture(&self, id: usize) -> Result<&RenderTexture, CoreError> {
        self.render_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Render Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_render_texture_mut(&mut self, id: usize) -> Result<&mut RenderTexture, CoreError> {
        self.render_textures
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Render Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_render_texture_ref(&self, id: usize) -> Result<Ref<RenderTexture>, CoreError> {
        let val = self
            .render_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Render Texture".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_render_texture(&mut self, id: usize) -> Result<RenderTexture, CoreError> {
        self.render_textures
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Render Texture".to_string(),
                id,
            ))
    }

    // Depth Texture
    #[inline]
    pub fn add_depth_texture(&mut self, rt: DepthTexture) {
        if self.depth_textures.contains_key(&rt.id) {
            warn!("Depth Texture with id: {} exist in `context`", rt.id);
        } else {
            let _ = self.ids.insert(rt.id);
            let _ = self.depth_textures.insert(rt.id, rt);
        }
    }

    #[inline]
    pub fn replace_depth_texture(
        &mut self,
        id: usize,
        mut rt: DepthTexture,
    ) -> Result<(), CoreError> {
        if self.depth_textures.contains_key(&id) {
            rt.id = id;
            *(self.get_depth_texture_mut(id)?) = rt;
        } else {
            warn!("Depth Texture with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_depth_texture(&self, id: usize) -> Result<&DepthTexture, CoreError> {
        self.depth_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Depth Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_depth_texture_mut(&mut self, id: usize) -> Result<&mut DepthTexture, CoreError> {
        self.depth_textures
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Depth Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_depth_texture_ref(&self, id: usize) -> Result<Ref<DepthTexture>, CoreError> {
        let val = self
            .depth_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Depth Texture".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_depth_texture(&mut self, id: usize) -> Result<DepthTexture, CoreError> {
        self.depth_textures
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Depth Texture".to_string(),
                id,
            ))
    }

    // Process Texture
    #[inline]
    pub fn add_process_texture(&mut self, rt: RenderTexture) {
        if self.process_textures.contains_key(&rt.id) {
            warn!("Process Texture with id: {} exist in `context`", rt.id);
        } else {
            let _ = self.ids.insert(rt.id);
            let _ = self.process_textures.insert(rt.id, rt);
        }
    }

    #[inline]
    pub fn replace_process_texture(
        &mut self,
        id: usize,
        mut rt: RenderTexture,
    ) -> Result<(), CoreError> {
        if self.process_textures.contains_key(&id) {
            rt.id = id;
            *(self.get_process_texture_mut(id)?) = rt;
        } else {
            warn!("Process Texture with id: {id} doesn't exist in `context`",);
        }

        Ok(())
    }

    #[inline]
    pub fn get_process_texture(&self, id: usize) -> Result<&RenderTexture, CoreError> {
        self.process_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Process Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_process_texture_mut(&mut self, id: usize) -> Result<&mut RenderTexture, CoreError> {
        self.process_textures
            .get_mut(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Process Texture".to_string(),
                id,
            ))
    }

    #[inline]
    pub fn get_process_texture_ref(&self, id: usize) -> Result<Ref<RenderTexture>, CoreError> {
        let val = self
            .process_textures
            .get(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Process Texture".to_string(),
                id,
            ))?;

        Ok(Ref::new(val))
    }

    #[inline]
    pub fn take_process_texture(&mut self, id: usize) -> Result<RenderTexture, CoreError> {
        self.process_textures
            .remove(&id)
            .ok_or(CoreError::ContextFieldIsNotExist(
                "Process Texture".to_string(),
                id,
            ))
    }
}
