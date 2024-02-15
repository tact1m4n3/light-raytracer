struct Settings {
    samples_per_render: u32,
    max_ray_depth: u32,
    furnace_test: u32,
}

struct PerRender {
    num_samples: u32,
    current_time: u32,
}

struct Camera {
    position: vec3<f32>,
    inverse_projection: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
}

struct Material {
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    emission: vec3<f32>,
}

struct Vertex {
    position: vec3<f32>,
    tex_coord: vec2<f32>,
    normal: vec3<f32>,
}

struct Triangle {
    vertex_indices: array<u32, 3>,
    material_index: u32,
}

const INF: f32 = 4294967296.0;
const PI: f32 = 3.1415926535897932384626433832795;
const EPSILON: f32 = 0.00001;

@group(0)
@binding(0)
var t_acc_input: texture_2d<f32>;

@group(0)
@binding(1)
var t_acc_output: texture_storage_2d<rgba32float, write>;

@group(0)
@binding(2)
var t_output: texture_storage_2d<rgba32float, write>;

@group(0)
@binding(3)
var<uniform> u_settings: Settings;

@group(0)
@binding(4)
var<uniform> u_per_render: PerRender;

@group(0)
@binding(5)
var<uniform> u_camera: Camera;

@group(0)
@binding(6)
var<storage, read> b_materials: array<Material>;

@group(0)
@binding(7)
var<storage, read> b_vertices: array<Vertex>;

@group(0)
@binding(8)
var<storage, read> b_triangles: array<Triangle>;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitPayload {
    hit_distance: f32,
    position: vec3<f32>,
    tex_coord: vec2<f32>,
    normal: vec3<f32>,
    material_index: u32,
}

fn per_pixel(coord: vec2<u32>) {
    var furnace_test = f32(clamp(u_settings.furnace_test, 0u, 1u));

    var acc_color: vec3<f32>;
    for (var i = 0u; i < u_settings.samples_per_render; i++) {
        var ray: Ray;
        ray.origin = u_camera.position;
        ray.direction = get_ray_direction(coord);

        var light = vec3<f32>(0.0);
        var contribution = vec3<f32>(1.0);

        for (var j = 0u; j < u_settings.max_ray_depth; j++) {
            var payload = trace_ray(ray);
            if payload.hit_distance < 0.0 {
                break;
            }

            var material = b_materials[payload.material_index];
            material.albedo = mix(material.albedo, vec3<f32>(1.0), furnace_test);

            light += contribution * material.emission;
            contribution *= material.albedo;

            ray.origin = payload.position;
            ray.direction = normalize(rand_unit_sphere() + payload.normal);
        }

        light += contribution * mix(vec3<f32>(0.1, 0.1, 0.3), vec3<f32>(1.0), furnace_test);

        acc_color += light;
    }

    acc_color += textureLoad(t_acc_input, coord, 0).xyz;

    var output_color = acc_color / f32(u_per_render.num_samples + u_settings.samples_per_render);
    output_color = aces_approx(output_color);
    output_color = pow(output_color, vec3<f32>(1. / 2.2));

    textureStore(t_acc_output, coord, vec4<f32>(acc_color, 1.0));
    textureStore(t_output, coord, vec4<f32>(output_color, 1.0));
}

fn trace_ray(ray: Ray) -> HitPayload {
    var hit_distance = INF;
    var triangle_index: u32;
    var hit_uv: vec2<f32>;

    for (var i: u32 = 0u; i < arrayLength(&b_triangles); i++) {
        var t: f32;
        var uv: vec2<f32>;
        if ray_triangle_intersection(ray, i, &t, &uv) && t > EPSILON && t < hit_distance {
            hit_distance = t;
            triangle_index = i;
            hit_uv = uv;
        }
    }

    if hit_distance == INF {
        return miss(ray);
    }

    return closest_hit(ray, hit_distance, triangle_index, hit_uv);
}

fn closest_hit(ray: Ray, hit_distance: f32, triangle_index: u32, uv: vec2<f32>) -> HitPayload {
    var payload: HitPayload;

    payload.hit_distance = hit_distance;

    let triangle = b_triangles[triangle_index];

    let v0i = triangle.vertex_indices[0];
    let v1i = triangle.vertex_indices[1];
    let v2i = triangle.vertex_indices[2];

    var tc0 = b_vertices[v0i].tex_coord;
    var tc1 = b_vertices[v1i].tex_coord;
    var tc2 = b_vertices[v2i].tex_coord;
    let tex_coord = (1.0 - uv.x - uv.y) * tc0 + uv.x * tc1 + uv.y * tc2;

    let n0 = b_vertices[v0i].normal;
    let n1 = b_vertices[v1i].normal;
    let n2 = b_vertices[v2i].normal;
    let normal = normalize((1.0 - uv.x - uv.y) * n0 + uv.x * n1 + uv.y * n2);

    payload.position = ray.origin + ray.direction * hit_distance;
    payload.tex_coord = tex_coord;
    payload.normal = normal;
    payload.material_index = triangle.material_index;

    return payload;
}

fn miss(ray: Ray) -> HitPayload {
    var payload: HitPayload;
    payload.hit_distance = -1.0;
    return payload;
}

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    var size = textureDimensions(t_output);
    if gid.x >= u32(size.x) || gid.y >= u32(size.y) {
        return;
    }

    rng_state = pcg_hash(pcg_hash(pcg_hash(gid.x) + gid.y) + u_per_render.current_time);
    per_pixel(gid.xy);
}

var<private> rng_state: u32;

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand(a: f32, b: f32) -> f32 {
    rng_state = pcg_hash(rng_state);
    return a + f32(rng_state) * (1.0 / 4294967296.0) * (b - a);
}

fn rand_vec2(a: f32, b: f32) -> vec2<f32> {
    return vec2<f32>(rand(a, b), rand(a, b));
}

fn rand_vec3(a: f32, b: f32) -> vec3<f32> {
    return vec3<f32>(rand(a, b), rand(a, b), rand(a, b));
}

fn rand_unit_sphere() -> vec3<f32> {
    return normalize(rand_vec3(-1.0, 1.0));
}

fn get_ray_direction(coord: vec2<u32>) -> vec3<f32> {
    var size = textureDimensions(t_output);
    var coord_unit = (vec2<f32>(coord) + rand_vec2(-0.5, 0.5)) / vec2<f32>(size);
    var final_coord = coord_unit * 2.0 - 1.0;
    final_coord.y = -final_coord.y; // flip the y coordinate
    var target_position = u_camera.inverse_projection * vec4<f32>(final_coord, 1.0, 1.0);
    var direction = (u_camera.inverse_view * vec4<f32>(normalize(target_position.xyz / target_position.w), 0.0)).xyz;
    return direction;
}

fn ray_triangle_intersection(ray: Ray, triangle_index: u32, t: ptr<function, f32>, uv: ptr<function, vec2<f32>>) -> bool {
    let triangle = b_triangles[triangle_index];

    let v0i = triangle.vertex_indices[0];
    let v1i = triangle.vertex_indices[1];
    let v2i = triangle.vertex_indices[2];

    var p0 = b_vertices[v0i].position;
    var p1 = b_vertices[v1i].position;
    var p2 = b_vertices[v2i].position;

    let p0p1 = p1 - p0;
    let p0p2 = p2 - p0;
    let pvec = cross(ray.direction, p0p2);
    let det = dot(p0p1, pvec);

    if det > -EPSILON && det < EPSILON {
        return false;
    }

    let inv_det = 1.0 / det;

    let tvec = ray.origin - p0;
    let u = dot(tvec, pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return false;
    }

    let qvec = cross(tvec, p0p1);
    let v = dot(ray.direction, qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    (*uv).x = u;
    (*uv).y = v;

    *t = dot(p0p2, qvec) * inv_det;

    return true;
}

fn aces_approx(x: vec3<f32>) -> vec3<f32> {
    var a = 2.51;
    var b = 0.03;
    var c = 2.43;
    var d = 0.59;
    var e = 0.14;
    return (x * (a * x + b)) / (x * (c * x + d) + e);
}
