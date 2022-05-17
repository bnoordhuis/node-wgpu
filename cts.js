"use strict"

const gpu = require("./")
exports.create = (flags) => gpu

gpu.GPUAdapter.prototype.features = new Set()
gpu.GPUDevice.prototype.lost = new Promise(() => {})
gpu.GPUDevice.prototype.pushErrorScope = () => {}
gpu.GPUDevice.prototype.popErrorScope = () => {}

globalThis.GPUBufferUsage = gpu.GPUBufferUsage
globalThis.GPUValidationError = class GPUValidationError extends Error {}
