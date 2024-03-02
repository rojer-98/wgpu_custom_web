use crate::{
    bind_group::{
        layout::{BindGroupLayout, BindGroupLayoutBuilder},
        BindGroup, BindGroupBuilder,
    },
    buffer::{Buffer, BufferBuilder},
    errors::CoreError,
    model::{Model, ModelBuilder},
    pipeline::{
        layout::{PipelineLayout, PipelineLayoutBuilder},
        Pipeline, PipelineBuilder,
    },
    shader::{Shader, ShaderBuilder},
    storage::{Storages, StoragesBuilder},
    texture::{DepthTexture, DepthTextureBuilder, RenderTexture, RenderTextureBuilder},
    traits::Builder,
    uniform::{Uniforms, UniformsBuilder},
    utils::Ref,
    worker::Worker,
};

impl<'a> Worker<'a> {
    // Foreign functions
    // Storages
    pub fn create_storage_id(&self) -> (usize, StoragesBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, StoragesBuilder::new_indexed(self.device, id))
    }

    pub fn create_storage(&self) -> StoragesBuilder<'_> {
        StoragesBuilder::new(self.device)
    }

    pub fn add_storage(&mut self, u: Storages) {
        self.context.add_storage(u)
    }

    pub fn replace_storage(&mut self, id: usize, u: Storages) -> Result<(), CoreError> {
        self.context.replace_storage(id, u)
    }

    pub fn get_storage(&self, id: usize) -> Result<&Storages, CoreError> {
        self.context.get_storage(id)
    }

    pub fn get_storage_mut(&mut self, id: usize) -> Result<&mut Storages, CoreError> {
        self.context.get_storage_mut(id)
    }

    pub fn get_storage_ref(&self, id: usize) -> Result<Ref<Storages>, CoreError> {
        self.context.get_storage_ref(id)
    }

    pub fn take_storage(&mut self, id: usize) -> Result<Storages, CoreError> {
        self.context.take_storage(id)
    }

    // Uniforms
    pub fn create_uniform_id(&self) -> (usize, UniformsBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, UniformsBuilder::new_indexed(self.device, id))
    }

    pub fn create_uniform(&self) -> UniformsBuilder<'_> {
        UniformsBuilder::new(self.device)
    }

    pub fn add_uniform(&mut self, u: Uniforms) {
        self.context.add_uniform(u)
    }

    pub fn replace_uniform(&mut self, id: usize, u: Uniforms) -> Result<(), CoreError> {
        self.context.replace_uniform(id, u)
    }

    pub fn get_uniform(&self, id: usize) -> Result<&Uniforms, CoreError> {
        self.context.get_uniform(id)
    }

    pub fn get_uniform_mut(&mut self, id: usize) -> Result<&mut Uniforms, CoreError> {
        self.context.get_uniform_mut(id)
    }

    pub fn get_uniform_ref(&self, id: usize) -> Result<Ref<Uniforms>, CoreError> {
        self.context.get_uniform_ref(id)
    }

    pub fn take_uniform(&mut self, id: usize) -> Result<Uniforms, CoreError> {
        self.context.take_uniform(id)
    }

    // Model
    pub fn create_model_id(&self) -> (usize, ModelBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, ModelBuilder::new_indexed(self.device, id))
    }

    pub fn create_model(&self) -> ModelBuilder<'_> {
        ModelBuilder::new(self.device)
    }

    pub fn add_model(&mut self, m: Model) {
        self.context.add_model(m)
    }

    pub fn replace_model(&mut self, id: usize, m: Model) -> Result<(), CoreError> {
        self.context.replace_model(id, m)
    }

    pub fn get_model(&self, id: usize) -> Result<&Model, CoreError> {
        self.context.get_model(id)
    }

    pub fn get_model_mut(&mut self, id: usize) -> Result<&mut Model, CoreError> {
        self.context.get_model_mut(id)
    }

    pub fn get_model_ref(&self, id: usize) -> Result<Ref<Model>, CoreError> {
        self.context.get_model_ref(id)
    }

    pub fn take_model(&mut self, id: usize) -> Result<Model, CoreError> {
        self.context.take_model(id)
    }

    // Buffer
    pub fn create_buffer_id<T: bytemuck::Zeroable + bytemuck::Pod>(
        &self,
    ) -> (usize, BufferBuilder<'_, T>) {
        let id = self.context.generate_unique_id();
        (id, BufferBuilder::new_indexed(self.device, id))
    }

    pub fn create_buffer<T: bytemuck::Zeroable + bytemuck::Pod>(&self) -> BufferBuilder<'_, T> {
        BufferBuilder::new(self.device)
    }

    pub fn add_buffer(&mut self, b: Buffer) {
        self.context.add_buffer(b)
    }

    pub fn replace_buffer(&mut self, id: usize, b: Buffer) -> Result<(), CoreError> {
        self.context.replace_buffer(id, b)
    }

    pub fn get_buffer(&self, id: usize) -> Result<&Buffer, CoreError> {
        self.context.get_buffer(id)
    }

    pub fn get_buffer_mut(&mut self, id: usize) -> Result<&mut Buffer, CoreError> {
        self.context.get_buffer_mut(id)
    }

    pub fn get_buffer_ref(&self, id: usize) -> Result<Ref<Buffer>, CoreError> {
        self.context.get_buffer_ref(id)
    }

    pub fn take_buffer(&mut self, id: usize) -> Result<Buffer, CoreError> {
        self.context.take_buffer(id)
    }

    // Bind group
    pub fn create_bind_group_layout_id(&self) -> (usize, BindGroupLayoutBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, BindGroupLayoutBuilder::new_indexed(self.device, id))
    }

    pub fn create_bind_group_layout(&self) -> BindGroupLayoutBuilder<'_> {
        BindGroupLayoutBuilder::new(self.device)
    }

    pub fn get_bind_group_layout(&self, id: usize) -> Result<&BindGroupLayout, CoreError> {
        self.context.get_bind_group_layout(id)
    }

    pub fn get_bind_group_layout_mut(
        &mut self,
        id: usize,
    ) -> Result<&mut BindGroupLayout, CoreError> {
        self.context.get_bind_group_layout_mut(id)
    }

    pub fn get_bind_group_layout_ref(&self, id: usize) -> Result<Ref<BindGroupLayout>, CoreError> {
        self.context.get_bind_group_layout_ref(id)
    }

    pub fn add_bind_group_layout(&mut self, bgl: BindGroupLayout) {
        self.context.add_bind_group_layout(bgl)
    }

    pub fn replace_bind_group_layout(
        &mut self,
        id: usize,
        bgl: BindGroupLayout,
    ) -> Result<(), CoreError> {
        self.context.replace_bind_group_layout(id, bgl)
    }

    pub fn take_bind_group_layout(&mut self, id: usize) -> Result<BindGroupLayout, CoreError> {
        self.context.take_bind_group_layout(id)
    }

    pub fn create_bind_group_id(&self) -> (usize, BindGroupBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, BindGroupBuilder::new_indexed(self.device, id))
    }

    pub fn create_bind_group(&self) -> BindGroupBuilder<'_> {
        BindGroupBuilder::new(self.device)
    }

    pub fn get_bind_group(&self, id: usize) -> Result<&BindGroup, CoreError> {
        self.context.get_bind_group(id)
    }

    pub fn get_bind_group_mut(&mut self, id: usize) -> Result<&mut BindGroup, CoreError> {
        self.context.get_bind_group_mut(id)
    }

    pub fn get_bind_group_ref(&self, id: usize) -> Result<Ref<BindGroup>, CoreError> {
        self.context.get_bind_group_ref(id)
    }

    pub fn add_bind_group(&mut self, bg: BindGroup) {
        self.context.add_bind_group(bg)
    }

    pub fn replace_bind_group(&mut self, id: usize, bg: BindGroup) -> Result<(), CoreError> {
        self.context.replace_bind_group(id, bg)
    }

    pub fn take_bind_group(&mut self, id: usize) -> Result<BindGroup, CoreError> {
        self.context.take_bind_group(id)
    }

    // Pipeline
    pub fn create_pipeline_layout_id(&self) -> (usize, PipelineLayoutBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, PipelineLayoutBuilder::new_indexed(self.device, id))
    }

    pub fn create_pipeline_layout(&self) -> PipelineLayoutBuilder<'_> {
        PipelineLayoutBuilder::new(self.device)
    }

    pub fn get_pipeline_layout(&self, id: usize) -> Result<&PipelineLayout, CoreError> {
        self.context.get_pipeline_layout(id)
    }

    pub fn get_pipeline_layout_mut(&mut self, id: usize) -> Result<&mut PipelineLayout, CoreError> {
        self.context.get_pipeline_layout_mut(id)
    }

    pub fn get_pipeline_layout_ref(&self, id: usize) -> Result<Ref<PipelineLayout>, CoreError> {
        self.context.get_pipeline_layout_ref(id)
    }

    pub fn add_pipeline_layout(&mut self, pl: PipelineLayout) {
        self.context.add_pipeline_layout(pl)
    }

    pub fn replace_pipeline_layout(
        &mut self,
        id: usize,
        pl: PipelineLayout,
    ) -> Result<(), CoreError> {
        self.context.replace_pipeline_layout(id, pl)
    }

    pub fn take_pipeline_layout(&mut self, id: usize) -> Result<PipelineLayout, CoreError> {
        self.context.take_pipeline_layout(id)
    }

    pub fn create_pipeline_id(&self) -> (usize, PipelineBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, PipelineBuilder::new_indexed(self.device, id))
    }

    pub fn create_pipeline(&self) -> PipelineBuilder<'_> {
        PipelineBuilder::new(self.device)
    }

    pub fn get_pipeline(&self, id: usize) -> Result<&Pipeline, CoreError> {
        self.context.get_pipeline(id)
    }

    pub fn get_pipeline_ref(&self, id: usize) -> Result<Ref<Pipeline>, CoreError> {
        self.context.get_pipeline_ref(id)
    }

    pub fn get_pipeline_mut(&mut self, id: usize) -> Result<&mut Pipeline, CoreError> {
        self.context.get_pipeline_mut(id)
    }

    pub fn add_pipeline(&mut self, p: Pipeline) {
        self.context.add_pipeline(p)
    }

    pub fn replace_pipeline(&mut self, id: usize, p: Pipeline) -> Result<(), CoreError> {
        self.context.replace_pipeline(id, p)
    }

    pub fn take_pipeline(&mut self, id: usize) -> Result<Pipeline, CoreError> {
        self.context.take_pipeline(id)
    }

    // Shader
    pub fn create_shader_id(&self) -> (usize, ShaderBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, ShaderBuilder::new_indexed(self.device, id))
    }

    pub fn create_shader(&self) -> ShaderBuilder<'_> {
        ShaderBuilder::new(self.device)
    }

    pub fn get_shader(&self, id: usize) -> Result<&Shader, CoreError> {
        self.context.get_shader(id)
    }

    pub fn get_shader_mut(&mut self, id: usize) -> Result<&mut Shader, CoreError> {
        self.context.get_shader_mut(id)
    }

    pub fn get_shader_ref(&self, id: usize) -> Result<Ref<Shader>, CoreError> {
        self.context.get_shader_ref(id)
    }

    pub fn add_shader(&mut self, sh: Shader) {
        self.context.add_shader(sh)
    }

    pub fn replace_shader(&mut self, id: usize, sh: Shader) -> Result<(), CoreError> {
        self.context.replace_shader(id, sh)
    }

    pub fn take_shader(&mut self, id: usize) -> Result<Shader, CoreError> {
        self.context.take_shader(id)
    }

    // Render texture
    pub fn create_render_texture_id(&self) -> (usize, RenderTextureBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, RenderTextureBuilder::new_indexed(self.device, id))
    }

    pub fn create_render_texture(&self) -> RenderTextureBuilder<'_> {
        RenderTextureBuilder::new(self.device)
    }

    pub fn get_render_texture(&self, id: usize) -> Result<&RenderTexture, CoreError> {
        self.context.get_render_texture(id)
    }

    pub fn get_render_texture_mut(&mut self, id: usize) -> Result<&mut RenderTexture, CoreError> {
        self.context.get_render_texture_mut(id)
    }

    pub fn get_render_texture_ref(&self, id: usize) -> Result<Ref<RenderTexture>, CoreError> {
        self.context.get_render_texture_ref(id)
    }

    pub fn add_render_texture(&mut self, rt: RenderTexture) {
        self.load_texture(&rt);
        self.context.add_render_texture(rt)
    }

    pub fn replace_render_texture(
        &mut self,
        id: usize,
        rt: RenderTexture,
    ) -> Result<(), CoreError> {
        self.context.replace_render_texture(id, rt)
    }

    pub fn take_render_texture(&mut self, id: usize) -> Result<RenderTexture, CoreError> {
        self.context.take_render_texture(id)
    }

    // Depth texture
    pub fn create_depth_texture_id(&self) -> (usize, DepthTextureBuilder<'_>) {
        let id = self.context.generate_unique_id();
        (id, DepthTextureBuilder::new_indexed(self.device, id))
    }

    pub fn create_depth_texture(&self) -> DepthTextureBuilder<'_> {
        DepthTextureBuilder::new(self.device)
    }

    pub fn get_depth_texture(&self, id: usize) -> Result<&DepthTexture, CoreError> {
        self.context.get_depth_texture(id)
    }

    pub fn get_depth_texture_mut(&mut self, id: usize) -> Result<&mut DepthTexture, CoreError> {
        self.context.get_depth_texture_mut(id)
    }

    pub fn get_depth_texture_ref(&self, id: usize) -> Result<Ref<DepthTexture>, CoreError> {
        self.context.get_depth_texture_ref(id)
    }

    pub fn add_depth_texture(&mut self, rt: DepthTexture) {
        self.context.add_depth_texture(rt)
    }

    pub fn replace_depth_texture(&mut self, id: usize, rt: DepthTexture) -> Result<(), CoreError> {
        self.context.replace_depth_texture(id, rt)
    }

    pub fn take_depth_texture(&mut self, id: usize) -> Result<DepthTexture, CoreError> {
        self.context.take_depth_texture(id)
    }

    // Process texture
}
