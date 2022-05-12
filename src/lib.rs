use napi::bindgen_prelude::*;
use napi_derive::napi;
use static_assertions::const_assert_eq;
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::rc::Rc;
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
                let format: wgpu::TextureFormat =
                    serde_plain::from_str(&target.format)
                        .map_err(into_napi_error)?;
                let target = wgpu::ColorTargetState::from(format);
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
        let pipeline = self.device.create_render_pipeline(&descriptor);
        Ok(GPURenderPipeline(Rc::new(pipeline)))
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

    #[napi]
    pub fn create_command_encoder(&self) -> GPUCommandEncoder {
        let descriptor = wgpu::CommandEncoderDescriptor { label: None };
        let encoder = self.device.create_command_encoder(&descriptor);
        let encoder = Box::leak(Box::new(encoder));
        GPUCommandEncoder(Rc::new(RefCell::new(Some(encoder))))
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
    pub bind_group_layouts: Vec<&'static GPUBindGroupLayout>,
    pub label: Option<String>,
}

#[napi]
pub struct GPUBindGroupLayout(wgpu::BindGroupLayout);

#[napi]
impl GPUBindGroupLayout {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }
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

#[napi]
pub struct GPURenderPipeline(Rc<wgpu::RenderPipeline>);

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

    #[napi]
    pub fn create_view(&self) -> GPUTextureView {
        let descriptor = wgpu::TextureViewDescriptor::default();
        GPUTextureView(self.0.create_view(&descriptor))
    }

    #[napi]
    pub fn destroy(&self) {
        self.0.destroy();
    }
}

#[napi]
pub struct GPUTextureView(wgpu::TextureView);

#[napi]
impl GPUTextureView {
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

#[napi]
pub struct GPUCommandEncoder(
    Rc<RefCell<Option<&'static mut wgpu::CommandEncoder>>>,
);

#[napi]
impl GPUCommandEncoder {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }

    #[napi]
    pub fn begin_render_pass(
        &mut self,
        descriptor: GPURenderPassDescriptor,
    ) -> napi::Result<GPURenderPassEncoder> {
        let mut color_attachments = vec![];
        for c in &descriptor.color_attachments {
            let c = wgpu::RenderPassColorAttachment::try_from(c)?;
            color_attachments.push(c);
        }
        let descriptor = wgpu::RenderPassDescriptor {
            label: None, // TODO
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };
        let rc = Rc::clone(&self.0);
        let encoder = rc
            .try_borrow_mut()
            .map_err(into_napi_error)?
            .take()
            .ok_or_else(|| into_napi_error("encoder taken"))?;
        let encoder = encoder as *mut wgpu::CommandEncoder;
        let render_pass =
            unsafe { &mut *encoder }.begin_render_pass(&descriptor);
        Ok(GPURenderPassEncoder {
            render_pass,
            encoder,
            rc: Some(rc),
            pipeline: None,
        })
    }

    #[napi]
    pub fn copy_texture_to_buffer(
        &mut self,
        source: GPUImageCopyTexture,
        dest: GPUImageCopyBuffer,
        size: GPUExtend3d,
    ) -> napi::Result<()> {
        let source = wgpu::ImageCopyTexture::try_from(&source)?;
        let dest = wgpu::ImageCopyBuffer::from(&dest);
        let size = wgpu::Extent3d::from(&size);
        self.0
            .try_borrow_mut()
            .map_err(into_napi_error)?
            .as_mut()
            .ok_or_else(|| into_napi_error("encoder taken"))?
            .copy_texture_to_buffer(source, dest, size);
        Ok(())
    }
}

impl Drop for GPUCommandEncoder {
    fn drop(&mut self) {
        let encoder = self.0.borrow_mut().take().expect("encoder taken");
        let encoder = unsafe { Box::from_raw(encoder) };
        drop(encoder);
    }
}

#[napi(object)]
pub struct GPUImageCopyBuffer {
    pub buffer: &'static GPUBuffer,
    pub offset: Option<u32>,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
}

impl<'a> From<&'a GPUImageCopyBuffer> for wgpu::ImageCopyBuffer<'a> {
    fn from(that: &GPUImageCopyBuffer) -> Self {
        let buffer = &that.buffer.0;
        let offset = that.offset.unwrap_or(0) as u64;
        let bytes_per_row = NonZeroU32::new(that.bytes_per_row);
        let rows_per_image = NonZeroU32::new(that.rows_per_image);
        let layout = wgpu::ImageDataLayout {
            offset,
            bytes_per_row,
            rows_per_image,
        };
        Self { buffer, layout }
    }
}

#[napi(object)]
pub struct GPUImageCopyTexture {
    pub texture: &'static GPUTexture,
    pub mip_level: Option<u32>,
    pub origin: Option<&'static GPUOrigin3d>,
    pub aspect: Option<String>,
}

impl<'a> TryFrom<&'a GPUImageCopyTexture> for wgpu::ImageCopyTexture<'a> {
    type Error = napi::Error;

    fn try_from(that: &GPUImageCopyTexture) -> napi::Result<Self> {
        let mip_level = that.mip_level.unwrap_or(0);

        let origin = that
            .origin
            .map(wgpu::Origin3d::from)
            .unwrap_or(wgpu::Origin3d::ZERO);

        let aspect = match that.aspect.as_deref().unwrap_or_default() {
            "" | "all" => wgpu::TextureAspect::All,
            "depth-only" => wgpu::TextureAspect::DepthOnly,
            "stencil-only" => wgpu::TextureAspect::StencilOnly,
            _ => return Err(into_napi_error("bad texture aspect")),
        };

        Ok(Self {
            texture: &that.texture.0,
            mip_level,
            origin,
            aspect,
        })
    }
}

#[napi]
pub struct GPUOrigin3d(napi::Either<GPUOrigin3dDict, Vec<u32>>);

#[derive(Default)]
#[napi(object)]
pub struct GPUOrigin3dDict {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl From<&GPUOrigin3d> for wgpu::Origin3d {
    fn from(that: &GPUOrigin3d) -> Self {
        match &that.0 {
            napi::Either::A(v) => v.into(),
            napi::Either::B(v) => {
                let x = v.get(0).copied().unwrap_or(0);
                let y = v.get(1).copied().unwrap_or(0);
                let z = v.get(2).copied().unwrap_or(0);
                Self { x, y, z }
            }
        }
    }
}

impl From<&GPUOrigin3dDict> for wgpu::Origin3d {
    fn from(that: &GPUOrigin3dDict) -> Self {
        let GPUOrigin3dDict { x, y, z } = *that;
        Self { x, y, z }
    }
}

#[napi(object)]
pub struct GPURenderPassDescriptor {
    pub label: Option<String>,
    pub color_attachments: Vec<GPURenderPassColorAttachment>,
}

#[napi]
pub struct GPURenderPassEncoder {
    render_pass: wgpu::RenderPass<'static>,
    encoder: *mut wgpu::CommandEncoder,
    rc: Option<Rc<RefCell<Option<&'static mut wgpu::CommandEncoder>>>>,
    pipeline: Option<Rc<wgpu::RenderPipeline>>,
}

#[napi]
impl GPURenderPassEncoder {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        not_a_constructor()
    }

    #[napi]
    pub fn set_pipeline(&'static mut self, pipeline: &GPURenderPipeline) {
        self.pipeline = Some(Rc::clone(&pipeline.0));
        self.render_pass
            .set_pipeline(self.pipeline.as_deref().unwrap())
    }

    #[napi]
    pub fn set_viewport(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        min_depth: f64,
        max_depth: f64,
    ) {
        self.render_pass.set_viewport(
            x as f32,
            y as f32,
            w as f32,
            h as f32,
            min_depth as f32,
            max_depth as f32,
        );
    }

    #[napi]
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: Option<u32>,
        first_vertex: Option<u32>,
        first_instance: Option<u32>,
    ) {
        let first_vertex = first_vertex.unwrap_or(0);
        let first_instance = first_instance.unwrap_or(0);
        let instance_count = instance_count.unwrap_or(1);
        let vertices = first_vertex..vertex_count;
        let instances = first_instance..instance_count;
        self.render_pass.draw(vertices, instances);
    }

    #[napi]
    pub fn end(&mut self) {
        if let Some(rc) = self.rc.take() {
            assert!(rc
                .borrow_mut()
                .replace(unsafe { &mut *self.encoder } as _)
                .is_none());
        }
    }
}

impl Drop for GPURenderPassEncoder {
    fn drop(&mut self) {
        self.end();
    }
}

#[napi(object)]
pub struct GPURenderPassColorAttachment {
    pub label: Option<String>,
    pub view: &'static GPUTextureView,
    pub load_op: Option<String>, // XXX required?
    pub store_op: String,
    pub clear_value: Option<&'static GPUColor>,
    pub resolve_target: Option<&'static GPUTextureView>,
}

impl TryFrom<&GPURenderPassColorAttachment>
    for wgpu::RenderPassColorAttachment<'static>
{
    type Error = napi::Error;

    fn try_from(that: &GPURenderPassColorAttachment) -> napi::Result<Self> {
        let clear_value =
            that.clear_value.map(wgpu::Color::from).unwrap_or_default();
        let load = match that.load_op.as_deref().unwrap_or_default() {
            "" | "load" => wgpu::LoadOp::Load,
            "clear" => wgpu::LoadOp::Clear(clear_value),
            _ => return Err(into_napi_error("bad load op")),
        };
        let store = match that.store_op.as_str() {
            "store" => true,
            "discard" => false,
            _ => return Err(into_napi_error("bad store op")),
        };
        Ok(Self {
            view: &that.view.0,
            resolve_target: that.resolve_target.map(|view| &view.0),
            ops: wgpu::Operations { load, store },
        })
    }
}

#[napi]
pub struct GPUColor(napi::Either<GPUColorDict, Vec<f64>>);

#[napi(object)]
pub struct GPUColorDict {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl From<&GPUColor> for wgpu::Color {
    fn from(that: &GPUColor) -> Self {
        match &that.0 {
            napi::Either::A(v) => v.into(),
            napi::Either::B(v) => {
                let r = v.get(0).copied().unwrap_or(0.0);
                let g = v.get(1).copied().unwrap_or(0.0);
                let b = v.get(2).copied().unwrap_or(0.0);
                let a = v.get(3).copied().unwrap_or(1.0);
                Self { r, g, b, a }
            }
        }
    }
}

impl From<&GPUColorDict> for wgpu::Color {
    fn from(that: &GPUColorDict) -> Self {
        let GPUColorDict { r, g, b, a } = *that;
        Self { r, g, b, a }
    }
}

fn not_a_constructor<T>() -> napi::Result<T> {
    Err(into_napi_error("not a constructor"))
}

fn into_napi_error(err: impl ToString) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}
