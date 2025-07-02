struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(2) @binding(0) var sampler_: sampler;
@group(2) @binding(1) var color_texture: texture_2d<f32>;
@group(2) @binding(2) var depth_texture: texture_depth_2d;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // Some very crude directional occlusion.
    let color_sample = textureSample(color_texture, sampler_, vertex.uv);
    let depth_sample = textureSample(depth_texture, sampler_, vertex.uv);
    let shade = pow(fwidth(depth_sample) * 1000.0, 0.5);
    let result = color_sample - vec4<f32>(shade);

    // Gamma correct the result for SRGB.
    return vec4<f32>(
        pow(result.r, 2.2),
        pow(result.g, 2.2),
        pow(result.b, 2.2),
        result.a,
    );
}

