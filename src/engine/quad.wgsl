@group(0) @binding(0) var<uniform> proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;
@group(1) @binding(0) var<uniform> tran: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
	var vertices: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
		vec2(-1.0, -1.0),
		vec2( 1.0, -1.0),
		vec2(-1.0,  1.0),
		vec2( 1.0,  1.0),
	);

	let v = vertices[index];
	return proj * view * tran * vec4(v.x, v.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
	return vec4<f32>(0.1, 0.5, 0.3, 1.0);
}