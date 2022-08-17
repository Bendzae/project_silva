#import bevy_pbr::mesh_view_bindings

// selection is 1.0 for "use this normal"
// and -1.0 for "use the negative of this normal"
// and 0.0 for "don't use this normal"
// the w component is whether to use absolute value or not
struct CustomMaterial {
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    intensity: f32,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // world_normal is a vector where each value is -1.0 to 1.0
    // where the vector represents the normal direction in world-space
    // world-space aligns with the global x,y,z axis
    // 
    // For example N.x will be 1.0 for faces pointing directly in
    // the positive x direction
    var Normal = normalize(world_normal);

    // return vec4(Normal, 1.0);
    // return vec4(values_to_show, 1.0);

    // The view vector. V is a unit vector pointing from the fragment
    // on the sphere toward the camera.
    var V = normalize(view.world_position.xyz - world_position.xyz);

    // The dot product returns the angle between N and V where 
    // fragments on the sphere that are pointing at the camera
    // (have the same angle as the V) are 1.0, faces perpendicular 
    // to V are 0.0, faces pointing away are -1.0. This is why we 
    // clamp the value here, to make sure we don't end up with 
    // negative numbers for NdotV.
    let NdotV = max(dot(Normal, V), 0.0001);
    
    // return vec4(vec3(NdotV), 1.0);

    // The fresnel value here is just the inverse of NdotV. 
    // So fragments pointing away will now be 1.0 and ones 
    // pointing at the camera will be 0.0
    var fresnel = clamp(1.0 - NdotV, 0.0, 1.0);

    // Here's were just increasing the contrast with pow 
    // and making it brighter by multiplying by 2
    fresnel = pow(fresnel, 3.0) * material.intensity;

    // return vec4(vec3(fresnel), 1.0);
    let a = material.color_a;
    let b = material.color_b;
    let color_a = vec3(a.x, a.y, a.z);
    let color_b = vec3(b.x, b.y, b.z);

    // let blue = vec3(0.0, 0.4, 0.5);
    // let green_blue = vec3(0.8, 0.802, 1.0);

    return vec4(mix(color_a, color_b, fresnel), 1.0);
}