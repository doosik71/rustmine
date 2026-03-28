struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(2.0, 0.0),
        vec2<f32>(0.0, 0.0)
    );
    var out: VsOut;
    out.position = vec4<f32>(positions[idx], 0.0, 1.0);
    out.uv = uvs[idx];
    return out;
}

struct PostUniform {
    blur: f32,
    _pad: vec3<f32>,
};

@group(0) @binding(0)
var scene_tex: texture_2d<f32>;
@group(0) @binding(1)
var scene_sampler: sampler;
@group(0) @binding(2)
var<uniform> post: PostUniform;

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let color = textureSample(scene_tex, scene_sampler, uv);
    if post.blur <= 0.01 {
        return color;
    }

    let tex_size = vec2<f32>(textureDimensions(scene_tex));
    let texel = 1.0 / tex_size;

    var sum = vec4<f32>(0.0);
    sum += textureSample(scene_tex, scene_sampler, uv + texel * vec2<f32>(-2.0, 0.0)) * 0.1;
    sum += textureSample(scene_tex, scene_sampler, uv + texel * vec2<f32>(-1.0, 0.0)) * 0.2;
    sum += textureSample(scene_tex, scene_sampler, uv) * 0.4;
    sum += textureSample(scene_tex, scene_sampler, uv + texel * vec2<f32>(1.0, 0.0)) * 0.2;
    sum += textureSample(scene_tex, scene_sampler, uv + texel * vec2<f32>(2.0, 0.0)) * 0.1;

    let blurred = sum;
    return mix(color, blurred, post.blur);
}
