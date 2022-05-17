node-wgpu
=========

A [WebGPU](https://gpuweb.github.io/gpuweb/) implementation based on the
excellent [wgpu](https://github.com/gfx-rs/wgpu) crate.

Work in progress. Just far along enough to execute the following snippet:

```js
// Adapted from https://github.com/denoland/webgpu-examples/blob/main/hello-triangle/mod.ts
import gpu from "gpu"

const {
    GPUBufferUsage,
    GPUTextureUsage,
} = gpu

const dimensions = { width: 200, height: 200 }

const adapter = await gpu.requestAdapter()
const device = await adapter?.requestDevice()

const shaderCode = `
    [[stage(vertex)]]
    fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
        let x = f32(i32(in_vertex_index) - 1);
        let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
        return vec4<f32>(x, y, 0.0, 1.0);
    }

    [[stage(fragment)]]
    fn fs_main() -> [[location(0)]] vec4<f32> {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
`

const shaderModule = device.createShaderModule({ code: shaderCode })

const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [],
})

const renderPipeline = device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
        module: shaderModule,
        entryPoint: "vs_main",
    },
    fragment: {
        module: shaderModule,
        entryPoint: "fs_main",
        targets: [
            { format: "rgba8unorm-srgb" },
        ],
    },
})

const { texture, outputBuffer } = createCapture(device, dimensions)

const encoder = device.createCommandEncoder()

const renderPass = encoder.beginRenderPass({
    colorAttachments: [{
        view: texture.createView(),
        storeOp: "store",
        loadValue: [0, 1, 0, 1],
    }],
})
renderPass.setPipeline(renderPipeline)
renderPass.draw(3, 1)
renderPass.end()

copyToBuffer(encoder, texture, outputBuffer, dimensions)

device.queue.submit([encoder.finish()])

function createCapture(device, dimensions) {
    const { padded } = getRowPadding(dimensions.width)
    const outputBuffer = device.createBuffer({
        label: "Capture",
        size: padded * dimensions.height,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    })
    const texture = device.createTexture({
        label: "Capture",
        size: dimensions,
        format: "rgba8unorm-srgb",
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    })
    return { outputBuffer, texture }
}

function getRowPadding(width) {
    // It is a webgpu requirement that BufferCopyView.layout.bytes_per_row % COPY_BYTES_PER_ROW_ALIGNMENT(256) == 0
    // So we calculate padded_bytes_per_row by rounding unpadded_bytes_per_row
    // up to the next multiple of COPY_BYTES_PER_ROW_ALIGNMENT.
    // https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
    const bytesPerPixel = 4
    const unpaddedBytesPerRow = width * bytesPerPixel
    const align = 256
    const paddedBytesPerRowPadding = (align - unpaddedBytesPerRow % align) % align
    const paddedBytesPerRow = unpaddedBytesPerRow + paddedBytesPerRowPadding

    return {
        unpadded: unpaddedBytesPerRow,
        padded: paddedBytesPerRow,
    }
}

function copyToBuffer(encoder, texture, buffer, dimensions) {
    const { padded } = getRowPadding(dimensions.width)
    encoder.copyTextureToBuffer(
        { texture },
        { buffer, bytesPerRow: padded, rowsPerImage: 0 },
        dimensions,
    )
}
```

notes
=====

To run the WebGPU conformance test suite:

    $ git clone https://github.com/gpuweb/cts
    $ cd cts
    $ npm install
    $ npm run standalone
    $ ./tools/run_node --gpu-provider /path/to/node-wgpu/cts.js webgpu:*
