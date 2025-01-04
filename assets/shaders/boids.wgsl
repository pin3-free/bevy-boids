
struct ShaderInput {
    positions: array<vec2f>
}

@group(0) @binding(0)
var<storage, read_write> input: ShaderInput;


@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
}
