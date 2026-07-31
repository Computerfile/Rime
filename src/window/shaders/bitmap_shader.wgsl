struct VertexInput {
	@location(0) pos : vec3<f32>,
	@location(1) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(1) uv: vec2<f32>,
}

@vertex
fn vertex_entry_point(model: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.pos = vec4f(model.pos.x, model.pos.y, model.pos.z, 1.0);
	out.uv = model.uv;
	
	return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4f {
    let texture = textureSample(t_diffuse, s_diffuse, in.uv);
	let alpha = texture.r;


    return vec4<f32>(alpha, alpha, alpha, alpha);
}

