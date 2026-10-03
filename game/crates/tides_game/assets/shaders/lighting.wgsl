// Darkness overlay: ambient gloom with pools cut out by point lights, then a
// screen-space vignette whose edges bleed purple as 污染 rises.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const MAX_LIGHTS: u32 = 32u;

struct Lighting {
    // xy = world position, z = radius (px), w = energy
    lights: array<vec4<f32>, 32>,
    // rgb = light colour
    colors: array<vec4<f32>, 32>,
    // rgb = colour of the dark, a = how dark (0 = off, 1 = pitch black)
    ambient: vec4<f32>,
    // x,y = viewport size (px), z = corruption 0..1, w = light count
    screen: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> u: Lighting;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.world_position.xy;
    var intensity = 0.0;
    var tint = vec3<f32>(0.0);
    let count = min(u32(u.screen.w), MAX_LIGHTS);
    for (var i = 0u; i < count; i = i + 1u) {
        let l = u.lights[i];
        let d = distance(p, l.xy) / l.z;
        // Soft quadratic falloff, zero at the radius.
        let k = clamp(1.0 - d, 0.0, 1.0);
        let e = k * k * l.w;
        intensity = intensity + e;
        tint = tint + u.colors[i].rgb * e;
    }
    let lit = clamp(intensity, 0.0, 1.0);
    let light_hue = tint / max(intensity, 0.0001);

    // Vignette from frag coords; corruption stains the edges violet.
    let uv = in.position.xy / u.screen.xy;
    let edge = smoothstep(0.35, 0.95, distance(uv, vec2<f32>(0.5)) * 1.35);
    let stain = vec3<f32>(0.35, 0.05, 0.45) * u.screen.z;

    let dark_rgb = mix(u.ambient.rgb, stain, edge * u.screen.z);
    let rgb = mix(dark_rgb, light_hue * 0.25, lit);
    let alpha = clamp(u.ambient.a * (1.0 - lit) + edge * 0.35, 0.0, 0.97);
    return vec4<f32>(rgb, alpha);
}
