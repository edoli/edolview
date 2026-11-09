// One 128-thread Welford/Chan reduction for min/max and population moments.
// Five vec4 arrays use 10 KiB of shared memory. Only finite pixels contribute.
struct Params { roi: vec4<u32>, image: vec4<u32>, operation: vec4<u32>, values: vec4<f32> }
struct Moment { lo: u32, hi: u32, count: u32, mean: f32, m2: f32 }
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<Moment>;
@group(0) @binding(3) var<uniform> params: Params;
var<workgroup> lows: array<vec4<u32>, 128>;
var<workgroup> highs: array<vec4<u32>, 128>;
var<workgroup> counts: array<vec4<u32>, 128>;
var<workgroup> means: array<vec4<f32>, 128>;
var<workgroup> m2s: array<vec4<f32>, 128>;
fn ordered(bits: vec4<u32>) -> vec4<u32> {
    return select(bits ^ vec4<u32>(0x80000000u), ~bits, (bits & vec4<u32>(0x80000000u)) != vec4<u32>(0u));
}
@compute @workgroup_size(128)
fn statistics(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>) {
    var lo = vec4<u32>(0xffffffffu);
    var hi = vec4<u32>(0u);
    var n = vec4<u32>(0u);
    var mean = vec4<f32>(0.0);
    var m2 = vec4<f32>(0.0);
    var index = gid.x;
    loop {
        if (index >= params.roi.z * params.roi.w) { break; }
        let pos = vec2<i32>(params.roi.xy + vec2<u32>(index % params.roi.z, index / params.roi.z));
        let bits = bitcast<vec4<u32>>(textureLoad(source, pos, 0));
        let valid = ((bits >> vec4<u32>(23u)) & vec4<u32>(255u)) != vec4<u32>(255u);
        lo = min(lo, select(vec4<u32>(0xffffffffu), ordered(bits), valid));
        hi = max(hi, select(vec4<u32>(0u), ordered(bits), valid));
        let value = select(vec4<f32>(0.0), bitcast<vec4<f32>>(bits), valid);
        n += select(vec4<u32>(0u), vec4<u32>(1u), valid);
        let delta = value - mean;
        mean += select(vec4<f32>(0.0), delta / vec4<f32>(max(n, vec4<u32>(1u))), valid);
        m2 += select(vec4<f32>(0.0), delta * (value - mean), valid);
        index += groups.x * 128u;
    }
    lows[lid.x] = lo; highs[lid.x] = hi; counts[lid.x] = n; means[lid.x] = mean; m2s[lid.x] = m2;
    workgroupBarrier();
    var offset = 64u;
    loop {
        if (offset == 0u) { break; }
        if (lid.x < offset) {
            let b = lid.x + offset;
            let na = counts[lid.x];
            let nb = counts[b];
            let total = na + nb;
            let delta = means[b] - means[lid.x];
            let weight = vec4<f32>(nb) / vec4<f32>(max(total, vec4<u32>(1u)));
            let new_mean = means[lid.x] + delta * weight;
            let new_m2 = m2s[lid.x] + m2s[b] + delta * delta * (vec4<f32>(na) * weight);
            means[lid.x] = select(select(new_mean, means[b], na == vec4<u32>(0u)), means[lid.x], nb == vec4<u32>(0u));
            m2s[lid.x] = select(select(new_m2, m2s[b], na == vec4<u32>(0u)), m2s[lid.x], nb == vec4<u32>(0u));
            counts[lid.x] = total;
            lows[lid.x] = min(lows[lid.x], lows[b]);
            highs[lid.x] = max(highs[lid.x], highs[b]);
        }
        workgroupBarrier();
        offset /= 2u;
    }
    if (lid.x == 0u) {
        for (var c = 0u; c < params.image.z; c++) {
            output[wid.x * 4u + c] = Moment(lows[0][c], highs[0][c], counts[0][c], means[0][c], m2s[0][c]);
        }
    }
}
