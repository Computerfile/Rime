
@vertex
fn vertex_entry_point(@location(0) pos : vec3f) -> @builtin(position) vec4f {
    return vec4f(pos.x, pos.y, pos.z, 1.0);
}

@fragment
fn fragment_main(@builtin(position) pos : vec4f) -> @location(0) vec4f {
	return vec4(0, 1, 0, 1);
}
