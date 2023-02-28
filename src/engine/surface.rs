use super::{Camera, Canvas, Mesh, Vertex};
use rhai::{Engine, AST, Dynamic, Array, Scope};

pub fn color_map(min: f32, max: f32, val: f32) -> [f32; 3] {
	let val = (val.min(max).max(min) - min) / (max - min);
	if val < 0.5 {
		[0.0, 1.0 - (val * 2.0), 1.0]
	} else {
		[(val - 0.5) * 2.0, 0.0, 1.0]
	}
}

pub fn donut(u: f32, v: f32, dr: f32, er: f32) -> Vec<f32> {
	vec![
		(dr + (er * u.cos())) * v.sin(),
		(dr + (er * u.cos())) * v.cos(),
		er * u.sin(),
	]
}

pub fn complex(u: f32, v: f32) -> Vec<f32> {
	let c = num_complex::Complex::new(u, v).powu(2);
	let color = color_map(-2.0, 2.0, c.im);
	[
		[u, v, c.re],
		color
	].concat()
}

pub fn default_fn(u: f32, v: f32) -> Vec<f32> {
	[
		[u, v, 0.0],
		color_map(-1.0, 1.0, glam::vec2(u, v).length().sin())
	].concat()
}

#[derive(Default)]
pub struct SurfaceConfig {
	u_min: f32,
	u_max: f32,
	v_min: f32,
	v_max: f32,
	u_segments: usize,
	v_segments: usize,
}

impl SurfaceConfig {
	fn from_scope(scope: &Scope) -> Self {
		Self::new(
			scope.get_value("u_min").unwrap(),
			scope.get_value("u_max").unwrap(),
			scope.get_value("v_min").unwrap(),
			scope.get_value("v_max").unwrap(),
			scope.get_value::<i64>("u_segments").unwrap() as _,
			scope.get_value::<i64>("v_segments").unwrap() as _,
		)
	}

	pub fn new(
		u_min: f32,
		u_max: f32,
		v_min: f32,
		v_max: f32,
		u_segments: usize,
		v_segments: usize,
	) -> Self {
		Self {
			u_min,
			u_max,
			v_min,
			v_max,
			u_segments,
			v_segments,
		}
	}

	fn generate_vertices(&self, mut f: impl FnMut(f32, f32) -> [f32; 6]) -> Vec<Vertex> {
		let Self {
			u_min, u_max,
			v_min, v_max,
			u_segments,
			v_segments,
		} = *self;

		let n_vertices = (u_segments + 1) * (v_segments + 1);
		let mut vertices = Vec::with_capacity(n_vertices);
		let note = "octagon face todo";
		dbg!(note);
		//let usre = u_segments * 2 + 1;
		//let vsre = v_segments * 2 + 1;
		let du = (u_max - u_min) / (u_segments as f32);
		let dv = (v_max - v_min) / (v_segments as f32);

		for (i, j) in itertools::iproduct!(0..=u_segments, 0..=v_segments) {
			let u = u_min + i as f32 * du;
			let v = v_min + j as f32 * dv;
			let [x, y, z, r, g, b] = f(u, v);
			vertices.push(Vertex {
				position: [x, y, z],
				color: [r, g, b],
			});
		}

		vertices
	}

	fn generate_indices(&self) -> Vec<u16> {
		let Self {
			u_segments,
			v_segments,
			..
		} = *self;

		let n_faces = u_segments * v_segments;
		let n_triangles = n_faces * 2;
		let n_indices = n_triangles * 3;

		let mut indices = Vec::with_capacity(n_indices);

		let n_vertices_per_row = v_segments + 1;

		for (i, j) in itertools::iproduct!(0..u_segments, 0..v_segments) {
			let idx0 = j + i * n_vertices_per_row;
			let idx1 = j + 1 + i * n_vertices_per_row;
			let idx2 = j + 1 + (i + 1) * n_vertices_per_row;
			let idx3 = j + (i + 1) * n_vertices_per_row;

			indices.push(idx0 as u16);
			indices.push(idx1 as u16);
			indices.push(idx2 as u16);

			indices.push(idx2 as u16);
			indices.push(idx3 as u16);
			indices.push(idx0 as u16);
		}

		indices
	}
}

pub struct Surface {
	pub mesh: Mesh,
	rhai: (Engine, AST),
}

impl Surface {
	pub fn new<P: Into<std::path::PathBuf>>(
		canvas: &Canvas,
		camera: &Camera,
		path: P,
	) -> Self
	{
		let mut engine = Engine::new();
		engine.register_fn("color_map", |min: f32, max: f32, val: f32| {
			let color = color_map(min, max, val);
			color.map(|c| Dynamic::from_float(c)).to_vec()
		});
		engine.register_fn("complex", |u: f32, v: f32| {
			complex(u, v).into_iter().map(|c| Dynamic::from_float(c)).collect::<Array>()
		});
		engine.register_fn("donut", |u: f32, v: f32, dr: f32, er: f32| {
			donut(u, v, dr, er).into_iter().map(|c| Dynamic::from_float(c)).collect::<Array>()
		});

		let ast = engine.compile_file(path.into()).unwrap();
		let mut scope = Scope::new();
		scope.push("u_min", -1.0f32);
		scope.push("u_max",  1.0f32);
		scope.push("v_min", -1.0f32);
		scope.push("v_max",  1.0f32);
		scope.push("u_segments", 100);
		scope.push("v_segments", 100);
		engine.run_ast_with_scope(&mut scope, &ast).unwrap();
		let config = SurfaceConfig::from_scope(&scope);

		let vertices = config.generate_vertices(|u: f32, v: f32| {
			engine.call_fn::<Dynamic>(&mut scope, &ast, "vertex", (u, v))
				.unwrap()
				.into_typed_array::<f32>()
				.unwrap()
				.try_into()
				.unwrap()
		});
		let indices = config.generate_indices();

		let mut mesh = Mesh::new(canvas, camera, vertices, indices);
		mesh.transform = glam::Mat4::from_cols_array(&engine
			.call_fn::<Dynamic>(&mut scope, &ast, "matrix", ())
			.unwrap()
			.into_typed_array::<f32>()
			.unwrap()
			.try_into()
			.unwrap()
		).transpose();

		Self {
			mesh,
			rhai: (engine, ast),
		}
	}

	pub fn update<P: Into<std::path::PathBuf>>(&mut self, canvas: &Canvas, camera: &Camera, path: P) {
		let (engine, ast) = &mut self.rhai;
		*ast = engine.compile_file(path.into()).unwrap();
		let mut scope = Scope::new();
		scope.push("u_min", -1.0f32);
		scope.push("u_max",  1.0f32);
		scope.push("v_min", -1.0f32);
		scope.push("v_max",  1.0f32);
		scope.push("u_segments", 100);
		scope.push("v_segments", 100);
		engine.run_ast_with_scope(&mut scope, &ast).unwrap();
		let config = SurfaceConfig::from_scope(&scope);
		let vertices = config.generate_vertices(|u: f32, v: f32| {
			engine.call_fn::<Dynamic>(&mut scope, &ast, "vertex", (u, v))
				.unwrap()
				.into_typed_array::<f32>()
				.unwrap()
				.try_into()
				.unwrap()
		});
		let indices = config.generate_indices();
		self.mesh = Mesh::new(canvas, camera, vertices, indices);
		self.mesh.transform = glam::Mat4::from_cols_array(&engine
			.call_fn::<Dynamic>(&mut scope, &ast, "matrix", ())
			.unwrap()
			.into_typed_array::<f32>()
			.unwrap()
			.try_into()
			.unwrap()
		).transpose();
	}
}