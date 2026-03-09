struct Uniforms {
    inverse_view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    settings: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var volume_texture: texture_3d<f32>;
@group(0) @binding(2)
var volume_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc_xy: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.ndc_xy = positions[vertex_index];
    return out;
}

fn intersect_box(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec2<f32> {
    let box_min = vec3<f32>(-1.0, -1.0, -1.0);
    let box_max = vec3<f32>(1.0, 1.0, 1.0);
    let inv_dir = 1.0 / ray_dir;
    let t0 = (box_min - ray_origin) * inv_dir;
    let t1 = (box_max - ray_origin) * inv_dir;
    let tsmaller = min(t0, t1);
    let tbigger = max(t0, t1);
    let t_enter = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
    let t_exit = min(min(tbigger.x, tbigger.y), tbigger.z);
    return vec2<f32>(t_enter, t_exit);
}

fn palette(value: f32) -> vec3<f32> {
    let t = clamp(value, 0.0, 1.0);
    let low = vec3<f32>(8.0 / 255.0, 12.0 / 255.0, 20.0 / 255.0);
    let mid = vec3<f32>(24.0 / 255.0, 128.0 / 255.0, 168.0 / 255.0);
    let high = vec3<f32>(246.0 / 255.0, 224.0 / 255.0, 94.0 / 255.0);

    if (t < 0.55) {
        return mix(low, mid, t / 0.55);
    }

    return mix(mid, high, (t - 0.55) / 0.45);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let clip_near = vec4<f32>(in.ndc_xy, 0.0, 1.0);
    let clip_far = vec4<f32>(in.ndc_xy, 1.0, 1.0);

    let world_near4 = uniforms.inverse_view_projection * clip_near;
    let world_far4 = uniforms.inverse_view_projection * clip_far;
    let world_near = world_near4.xyz / world_near4.w;
    let world_far = world_far4.xyz / world_far4.w;

    let ray_origin = uniforms.camera_position.xyz;
    let ray_dir = normalize(world_far - world_near);
    let hit = intersect_box(ray_origin, ray_dir);

    if (hit.x > hit.y) {
        return vec4<f32>(0.02, 0.03, 0.05, 1.0);
    }

    let t_start = max(hit.x, 0.0);
    let t_end = hit.y;
    let step_count = max(u32(uniforms.settings.z), 8u);
    let step_size = (t_end - t_start) / f32(step_count);

    var accumulated = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var t = t_start;

    for (var i: u32 = 0u; i < step_count; i = i + 1u) {
        let world_position = ray_origin + ray_dir * (t + step_size * 0.5);
        let tex_coord = world_position * 0.5 + vec3<f32>(0.5, 0.5, 0.5);
        let sample_value = textureSampleLevel(volume_texture, volume_sampler, tex_coord, 0.0).r;
        let visible = max(sample_value - uniforms.settings.x, 0.0);
        if (visible > 0.0) {
            let normalized = visible / max(1.0 - uniforms.settings.x, 1e-5);
            let alpha = 1.0 - exp(-normalized * uniforms.settings.y * step_size * 6.0);
            let color = palette(sample_value);
            let remaining = 1.0 - accumulated.a;
            let next_rgb = accumulated.rgb + remaining * alpha * color;
            let next_alpha = accumulated.a + remaining * alpha;
            accumulated = vec4<f32>(next_rgb, next_alpha);
            if (accumulated.a > 0.985) {
                break;
            }
        }
        t = t + step_size;
    }

    let background = vec3<f32>(0.02, 0.03, 0.05);
    let rgb = accumulated.rgb + background * (1.0 - accumulated.a);
    return vec4<f32>(rgb, 1.0);
}
