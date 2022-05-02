use napi_derive::napi;
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

fn not_a_constructor<T>() -> napi::Result<T> {
    Err(into_napi_error("not a constructor"))
}

fn into_napi_error(err: impl ToString) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}
