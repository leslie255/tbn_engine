struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(2) @binding(0) var<uniform> fill_color: vec4<f32>;
@group(2) @binding(1) var<uniform> center: vec2<f32>;
@group(2) @binding(2) var<uniform> radius: f32;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let sd = distance(vertex.uv.xy, center) - radius;
    let aaf = fwidth(vertex.uv.x);
    let aaf_half = aaf * 0.5;
    let alpha = smoothstep(-aaf_half, aaf_half, -sd);
    return vec4<f32>(fill_color.rgb, fill_color.a * alpha);
}
