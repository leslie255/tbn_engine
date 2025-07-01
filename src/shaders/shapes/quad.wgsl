struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0) var<uniform> model_view: mat4x4<f32>;
@group(1) @binding(1) var<uniform> uv_transform: mat4x4<f32>;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var result: VertexOutput;
    result.uv = (uv_transform * vec4<f32>(position.x, 1.0 - position.y, 0.0, 0.0)).xy;
    result.position = projection * model_view * vec4<f32>(position.xy, 0.0, 1.0);
    return result;
}
