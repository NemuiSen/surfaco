u_min = -PI();
u_max =  PI();
v_min = -PI();
v_max =  PI();
u_segments = 25;
v_segments = 25;

fn matrix() {
	[
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 1.0, 0.0,
	0.0, 0.0, 0.0, 1.0,
	]
}

fn vertex(u, v) {
	donut(u, v, 1.0, 0.5) + color_map(-1.0, 1.0, sin(v*5))
	//complex(u, v)
}