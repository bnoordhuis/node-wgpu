node-wgpu
=========

A [WebGPU](https://gpuweb.github.io/gpuweb/) implementation based on the
excellent [wgpu](https://github.com/gfx-rs/wgpu) crate.

Work in progress. Just far along enough to execute the following snippet:

```js
// Adapted from https://github.com/denoland/webgpu-examples/blob/main/hello-triangle/mod.ts
import gpu from "gpu"

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
```
