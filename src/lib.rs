use napi::bindgen_prelude::ToNapiValue;
use napi_derive::napi;
use static_assertions::const_assert_eq;
use std::sync::Arc;

#[napi]
pub async fn request_adapter() -> Option<GPUAdapter> {
    let backends = wgpu::Backends::all();
    let instance = wgpu::Instance::new(backends);
    let options = wgpu::RequestAdapterOptions::default();
    instance
        .request_adapter(&options)
        .await
        .map(Arc::new)
        .map(GPUAdapter)
}

#[napi]
pub struct GPUAdapter(Arc<wgpu::Adapter>);

#[napi]
impl GPUAdapter {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }

    #[napi(getter)]
    pub fn get_name(&self) -> String {
        self.0.get_info().name
    }

    #[napi(getter)]
    pub fn get_is_fallback_adapter(&self) -> bool {
        false // TODO
    }

    #[napi]
    pub async fn request_device(&self) -> napi::Result<GPUDevice> {
        let descriptor = wgpu::DeviceDescriptor::default();
        self.0
            .request_device(&descriptor, None)
            .await
            .map_err(into_napi_error)
            .map(|(device, queue)| GPUDevice { device, queue })
    }
}

#[napi]
pub struct GPUDevice {
    device: wgpu::Device,
    #[allow(dead_code)]
    queue: wgpu::Queue,
}

#[napi]
impl GPUDevice {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }

    #[napi]
    pub fn create_shader_module(
        &self,
        descriptor: GPUShaderModuleDescriptor,
    ) -> GPUShaderModule {
        let label = descriptor.label.as_deref();
        let source = wgpu::ShaderSource::Wgsl(descriptor.code.into());
        let descriptor = wgpu::ShaderModuleDescriptor { label, source };
        GPUShaderModule(self.device.create_shader_module(&descriptor))
    }

    #[napi]
    pub fn create_pipeline_layout(
        &self,
        descriptor: GPUPipelineLayoutDescriptor,
    ) -> GPUPipelineLayout {
        let label = descriptor.label.as_deref();
        let descriptor = wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts: &[], // TODO
            push_constant_ranges: &[],
        };
        GPUPipelineLayout(self.device.create_pipeline_layout(&descriptor))
    }

    #[napi]
    pub fn create_render_pipeline(
        &self,
        descriptor: GPURenderPipelineDescriptor,
    ) -> napi::Result<GPURenderPipeline> {
        let label = descriptor.label.as_deref();
        let layout = descriptor.layout.map(|layout| &layout.0);
        let vertex = wgpu::VertexState {
            module: &descriptor.vertex.module.0,
            entry_point: &descriptor.vertex.entry_point,
            buffers: &[], // TODO
        };
        let mut fragment_targets = vec![];
        let fragment = if let Some(fragment) = &descriptor.fragment {
            for target in &fragment.targets {
                let target = wgpu::ColorTargetState::try_from(target)
                    .map_err(into_napi_error)?;
                fragment_targets.push(target);
            }
            Some(wgpu::FragmentState {
                module: &fragment.module.0,
                entry_point: &fragment.entry_point,
                targets: &fragment_targets,
            })
        } else {
            None
        };
        let multisample = wgpu::MultisampleState::default();
        let primitive = wgpu::PrimitiveState::default();
        let descriptor = wgpu::RenderPipelineDescriptor {
            label,
            layout,
            vertex,
            fragment,
            multisample,
            primitive,
            depth_stencil: None,
            multiview: None,
        };
        Ok(GPURenderPipeline(
            self.device.create_render_pipeline(&descriptor),
        ))
    }

    #[napi]
    pub fn create_buffer(
        &self,
        descriptor: GPUBufferDescriptor,
    ) -> napi::Result<GPUBuffer> {
        let label = descriptor.label.as_deref();
        let usage = wgpu::BufferUsages::from_bits(descriptor.usage)
            .ok_or_else(|| into_napi_error("bad BufferUsage"))?;
        let descriptor = wgpu::BufferDescriptor {
            label,
            usage,
            size: descriptor.size.into(),
            mapped_at_creation: descriptor.mapped_at_creation.unwrap_or(false),
        };
        Ok(GPUBuffer(self.device.create_buffer(&descriptor)))
    }

    #[napi]
    pub fn create_texture(
        &self,
        descriptor: GPUTextureDescriptor,
    ) -> napi::Result<GPUTexture> {
        let label = descriptor.label.as_deref();
        let size = wgpu::Extent3d::from(&descriptor.size);
        let mip_level_count = descriptor.mip_level_count.unwrap_or(1);
        let sample_count = descriptor.sample_count.unwrap_or(1);
        let dimension = match descriptor.dimension.as_deref() {
            Some("1d") => wgpu::TextureDimension::D1,
            Some("2d") | None => wgpu::TextureDimension::D2,
            Some("3d") => wgpu::TextureDimension::D3,
            _ => return Err(into_napi_error("bad texture dimension")),
        };
        let format =
            serde_plain::from_str::<wgpu::TextureFormat>(&descriptor.format)
                .map_err(into_napi_error)?;
        let usage = wgpu::TextureUsages::from_bits(descriptor.usage)
            .ok_or_else(|| into_napi_error("bad texture usage"))?;
        let descriptor = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage,
        };
        Ok(GPUTexture(self.device.create_texture(&descriptor)))
    }
}

#[napi(object)]
pub struct GPUShaderModuleDescriptor {
    pub code: String,
    pub label: Option<String>,
}

#[napi]
pub struct GPUShaderModule(wgpu::ShaderModule);

#[napi]
impl GPUShaderModule {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
}

#[napi(object)]
pub struct GPUPipelineLayoutDescriptor {
    pub bind_group_layouts: Vec<()>, // TODO Vec<GPUBindGroupLayout>
    pub label: Option<String>,
}

#[napi]
pub struct GPUPipelineLayout(wgpu::PipelineLayout);

#[napi]
impl GPUPipelineLayout {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
}

#[napi(object)]
pub struct GPURenderPipelineDescriptor {
    pub label: Option<String>,
    pub layout: Option<&'static GPUPipelineLayout>,
    pub vertex: GPUVertexState,
    pub fragment: Option<GPUFragmentState>,
}

#[napi(object)]
pub struct GPUVertexState {
    pub module: &'static GPUShaderModule,
    pub entry_point: String,
}

#[napi(object)]
pub struct GPUFragmentState {
    pub module: &'static GPUShaderModule,
    pub entry_point: String,
    pub targets: Vec<GPUColorTargetState>,
}

#[napi(object)]
pub struct GPUColorTargetState {
    pub format: String,
}

impl TryFrom<&GPUColorTargetState> for wgpu::ColorTargetState {
    type Error = serde_plain::Error;

    fn try_from(target: &GPUColorTargetState) -> Result<Self, Self::Error> {
        serde_plain::from_str::<wgpu::TextureFormat>(&target.format)
            .map(wgpu::ColorTargetState::from)
    }
}

#[napi]
pub struct GPURenderPipeline(wgpu::RenderPipeline);

#[napi]
impl GPURenderPipeline {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
}

// TODO napi-rs won't let us alias or refer to wgpu::BindUsages::* here
#[allow(non_camel_case_types)]
#[repr(u32)]
#[napi]
pub enum GPUBufferUsage {
    MAP_READ = 1,
    MAP_WRITE = 2,
    COPY_SRC = 4,
    COPY_DST = 8,
    INDEX = 16,
    VERTEX = 32,
    UNIFORM = 64,
    STORAGE = 128,
    INDIRECT = 256,
}

#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::MAP_READ as u32, wgpu::BufferUsages::MAP_READ.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::MAP_WRITE as u32, wgpu::BufferUsages::MAP_WRITE.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::COPY_SRC as u32, wgpu::BufferUsages::COPY_SRC.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::COPY_DST as u32, wgpu::BufferUsages::COPY_DST.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::INDEX as u32, wgpu::BufferUsages::INDEX.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::VERTEX as u32, wgpu::BufferUsages::VERTEX.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::UNIFORM as u32, wgpu::BufferUsages::UNIFORM.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::STORAGE as u32, wgpu::BufferUsages::STORAGE.bits());
#[rustfmt::skip] const_assert_eq!(GPUBufferUsage::INDIRECT as u32, wgpu::BufferUsages::INDIRECT.bits());

#[napi(object)]
pub struct GPUBufferDescriptor {
    pub label: Option<String>,
    pub size: u32, // TODO should be u64 but napi-rs won't let us
    pub usage: u32,
    pub mapped_at_creation: Option<bool>,
}

#[napi]
pub struct GPUBuffer(wgpu::Buffer);

#[napi]
impl GPUBuffer {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
}

#[napi]
pub struct GPUTexture(wgpu::Texture);

#[napi]
impl GPUTexture {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
}

#[napi(object)]
pub struct GPUTextureDescriptor {
    pub label: Option<String>,
    pub size: GPUExtend3d,
    pub format: String,
    pub mip_level_count: Option<u32>,
    pub sample_count: Option<u32>,
    pub dimension: Option<String>,
    pub usage: u32,
}

#[napi(object)]
pub struct GPUExtend3d {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: Option<u32>,
}

impl From<&GPUExtend3d> for wgpu::Extent3d {
    fn from(that: &GPUExtend3d) -> Self {
        Self {
            width: that.width,
            height: that.height,
            depth_or_array_layers: that.depth_or_array_layers.unwrap_or(1),
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[napi]
pub enum GPUTextureUsage {
    COPY_SRC = 1,
    COPY_DST = 2,
    TEXTURE_BINDING = 4,
    STORAGE_BINDING = 8,
    RENDER_ATTACHMENT = 16,
}

#[rustfmt::skip] const_assert_eq!(GPUTextureUsage::COPY_SRC as u32, wgpu::TextureUsages::COPY_SRC.bits());
#[rustfmt::skip] const_assert_eq!(GPUTextureUsage::COPY_DST as u32, wgpu::TextureUsages::COPY_DST.bits());
#[rustfmt::skip] const_assert_eq!(GPUTextureUsage::TEXTURE_BINDING as u32, wgpu::TextureUsages::TEXTURE_BINDING.bits());
#[rustfmt::skip] const_assert_eq!(GPUTextureUsage::STORAGE_BINDING as u32, wgpu::TextureUsages::STORAGE_BINDING.bits());
#[rustfmt::skip] const_assert_eq!(GPUTextureUsage::RENDER_ATTACHMENT as u32, wgpu::TextureUsages::RENDER_ATTACHMENT.bits());

fn not_a_constructor<T>() -> napi::Result<T> {
    Err(into_napi_error("not a constructor"))
}

fn into_napi_error(err: impl ToString) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}
