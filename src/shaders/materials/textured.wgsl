struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var sampler_: sampler;
@group(2) @binding(2) var<uniform> gamma: f32;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(texture, sampler_, vertex.uv);
    return vec4<f32>(
        pow(sample.x, gamma),
        pow(sample.y, gamma),
        pow(sample.z, gamma),
        sample.a,
    );
}
