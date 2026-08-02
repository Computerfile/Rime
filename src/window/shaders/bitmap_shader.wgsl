struct InstanceInput {
    @location(2) offset: vec2<f32>,
    @location(3) uv_min: vec2<f32>,
    @location(4) uv_max: vec2<f32>,
    @location(5) scaleX: f32,
	@location(6) y_bounds: vec2<f32>,
}

struct VertexInput {
	@location(0) pos : vec3<f32>,
	@location(1) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) pos: vec4<f32>,
	@location(1) uv: vec2<f32>,
}

@vertex
fn vertex_entry_point(model: VertexInput, instance: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	let is_top = model.pos.y > 0;
	let y_local = select(instance.y_bounds.y, instance.y_bounds.x, is_top);
	let x_local = model.pos.x * instance.scaleX;
    out.pos = vec4f(instance.offset.x + x_local, instance.offset.y + y_local, model.pos.z, 1.0);
	out.uv = instance.uv_min + model.uv * (instance.uv_max - instance.uv_min);
	
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
	// return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

