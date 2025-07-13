struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(2) @binding(0) var sampler_: sampler;
@group(2) @binding(1) var color_texture: texture_2d<f32>;
@group(2) @binding(2) var depth_texture: texture_depth_2d;
@group(2) @binding(3) var<uniform> near: f32;
@group(2) @binding(4) var<uniform> far: f32;

fn linearize_depth(depth: f32) -> f32 {
    let z: f32 = depth * 2.0 - 1.0; // back to NDC 
    return (2.0 * near * far) / (far + near - z * (far - near));	
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color_sample = textureSample(color_texture, sampler_, vertex.uv);
    let depth_sample = linearize_depth(textureSample(depth_texture, sampler_, vertex.uv));

    // let result = vec4<f32>(vec3<f32>(1.0 - depth_sample / (far - near)), 1.0);
    // let result = color_sample;

    let highlight = fwidth(depth_sample) / (far - near) * 40.0;
    let result = vec4<f32>(color_sample.rgb - vec3(highlight), color_sample.a);

    // Gamma correct the result for SRGB.
    return vec4<f32>(
        pow(result.r, 2.2),
        pow(result.g, 2.2),
        pow(result.b, 2.2),
        result.a,
    );
}

