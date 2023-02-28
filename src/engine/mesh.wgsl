struct VertexInput {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
}

struct FragmentInput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;
@group(1) @binding(0) var<uniform> tran: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> FragmentInput {
	var output: FragmentInput;
	output.position = proj * view * tran * vec4<f32>(input.position, 1.0);
	output.color = input.color;
	return output;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
	var r = input.color.r;
	var g = input.color.g;
	var b = input.color.b;
	var a = 0.5;
	var z = input.position.z;
	var weight = max(min(1.0, max(max(r, g), b) * a), a) * z;
	return vec4<f32>(input.color * a, a) * weight;
}