use thiserror::*;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("data in `{0}` is not set")]
    EmptyData(String),
    #[error("index data in `{0}` is not set")]
    EmptyIndexData(String),
    #[error("entries in `{0}` is not set")]
    EmptyEntries(String),
    #[error("binding in `{0}` is not set")]
    EmptyBinding(String),
    #[error("bind type in `{0}` is not set")]
    EmptyBindType(String),
    #[error("layout in `{0}` is not set")]
    EmptyLayout(String),
    #[error("entry point in `{0}` is not set")]
    EmptyEntryPoint(String),
    #[error("shader kind in `{0}` is not set")]
    EmptyShaderKind(String),
    #[error("shader source in `{0}` is not set")]
    EmptyShaderSource(String),
    #[error("fragment shader options in `{0}` is not set")]
    EmptyFragmentOptions(String),
    #[error("vertex shader options in `{0}` is not set")]
    EmptyVertexOptions(String),
    #[error("shader kind in `{0}` is wrong")]
    WrongShaderKind(String),
    #[error("pipeline multisample in `{0}` is not set")]
    EmptyPipelineMultisample(String),
    #[error("pipeline primitive in `{0}` is not set")]
    EmptyPipelinePrimitive(String),
    #[error("pipeline vertex state in `{0}` is not set")]
    EmptyPipelineVertex(String),
    #[error("size in `{0}` is not set")]
    EmptyTextureSize(String),
    #[error("texture view in `{0}` is not set")]
    EmptyTextureView(String),
    #[error("texture sampler in `{0}` is not set")]
    EmptyTextureSampler(String),
    #[error("diffuse texture in `{0}` is not set")]
    EmptyDiffuseTexture(String),
    #[error("normal texture in `{0}` is not set")]
    EmptyNormalTexture(String),
    #[error("bind group in `{0}` is not set")]
    EmptyBindGroup(String),
    #[error("bind group layout in `{0}` is not set")]
    EmptyBindGroupLayout(String),
    #[error("color attachments in `{0}` is not set")]
    EmptyRenderPassColorAttachemnts(String),
    #[error("query type in `{0}` is not set")]
    EmptyQueryType(String),
    #[error("{0} with id: {1} is not exixt in `context`")]
    ContextFieldIsNotExist(String, usize),
    #[error("cannot create image buffer")]
    ImageBufferCreate,
    #[error("obj file in `{0} is not set`")]
    EmptyObjFile(String),
    #[error("uniform buffer `{0}` isn't found")]
    UniformBufferNotFound(String),
    #[error("storage `{0}` isn't found")]
    StorageNotFound(String),
    #[error("surface properties is not configured")]
    SurfaceNotConfigured,
    #[error("view of worker is not init")]
    NotInitView,
    #[error("data is more than buffer size")]
    WrongBufferSize,

    // foreign errors
    #[error(transparent)]
    BufferAsyncError(#[from] wgpu::BufferAsyncError),
    #[error(transparent)]
    FlumeRecvError(#[from] flume::RecvError),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error(transparent)]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    TobjError(#[from] tobj::LoadError),
}
