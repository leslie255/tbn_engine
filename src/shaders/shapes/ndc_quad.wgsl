struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var result: VertexOutput;
    result.uv = vec2<f32>(position.x, 1.0 - position.y);
    result.position = vec4<f32>(position.xy, 0.0, 0.0);
    return result;
}
