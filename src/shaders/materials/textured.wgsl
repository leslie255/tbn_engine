struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var sampler_: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler_, vertex.uv);
}
