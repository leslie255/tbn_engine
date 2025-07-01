struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0) var<uniform> model_view: mat4x4<f32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var result: VertexOutput;
    result.uv = uv;
    result.position = projection * model_view * vec4<f32>(position.xyz, 1.0);
    return result;
}

