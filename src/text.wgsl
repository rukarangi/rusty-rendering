// VERTEX

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main (in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(in.position, 1.0);
    out.tex_coords = in.tex_coords;

    return out;
}

// FRAGMENT

@group(0)@binding(0)
var t_sheet: texture_2d<f32>;
@group(0)@binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(t_sheet, t_sampler, in.tex_coords);
    //let color: vec4<f32> = vec4(1.0, 1.0, 0.0, 1.0);

    let a: f32  = color.w;

    var color_out: vec3<f32> = vec3(0.0, 0.0, 0.0);

    if (a > 0.0) {
        color_out = vec3(1.0, 1.0, 1.0);
    }
    if (a == 0.0) {
        color_out = vec3(1.0, 0.0, 0.0);
    }

    return vec4<f32>(color_out, 0.5);
    //return color;
}