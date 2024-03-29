// VERTEX

struct CameraUniform {
    view_proj: mat4x4<f32>,
    pos: vec2<f32>,
}

@group(1)@binding(0)
var<uniform> camera: CameraUniform;

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
    out.clip_position = (camera.view_proj * vec4(in.position - vec3(camera.pos, 0.0), 1.0));
    // this one will move stuff basef on camera position, final use for all but ui

    // out.clip_position = (camera.view_proj * vec4(in.position , 1.0));
    // this one ignores camera position, final use for ui

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

    //let a: f32  = color.w;

    //var color_out: vec3<f32> = vec3(0.0, 0.0, 0.0);

    //if (a > 0.0) {
        //color_out = vec3(in.clip_position.xy, 1.0);
    //}
    //if (a == 0.0) {
    //    color_out = vec3(1.0, 0.0, 0.0);
    //}

    //return vec4<f32>(color_out, 0.5);
    //return vec4<f32>(in.clip_position.x,0.0,0.0, 1.0);
    return color;
}