
#include "common.hlsli"

#ifdef COMPUTE

cbuffer Params : register(b0) {
    int4 Rect;
};

RWBuffer<uint> CountBuf : register(u0);

#define THREAD 8

[numthreads(THREAD, THREAD, 1)]
void ColorCloudCs(uint2 id: SV_DispatchThreadID) {
    uint2 position = Rect.xy + id;
    if (all(position < Rect.zw)) {
        float3 color = Desktop[position].rgb;
        uint color_code = RgbToInt(color);

        uint4 same_color_lanes_mask = WaveMatch(color_code);
        if (WaveMultiPrefixCountBits(true, same_color_lanes_mask) == 0) { // first lane that is this color code
            uint4 counts = countbits(same_color_lanes_mask);
            InterlockedAdd(CountBuf[color_code], counts.x);
        }
    } 
}

#endif // COMPUTE

#ifdef GRAPHICS

cbuffer Params : register(b0) {
    float4x3 Projection;
    uint MinCount;
    float InvMaxCount;
    uint ColorSpace;
};

Buffer<uint> CountBuf : register(t0);

#define GRID 8
#define ELEMS 32
#define STEPS 2

struct Payload {
    uint counts_and_indices[GRID * GRID * GRID];
    uint num;
    uint base_color_code;
};

groupshared Payload payload;
groupshared uint nonzeros_in_group;

[numthreads(4, 4, 4)]
void ColorCloudAs(uint tid: SV_DispatchThreadID, uint3 gid: SV_GroupID, uint tig: SV_GroupIndex) {
    if (tig == 0) {
        nonzeros_in_group = 0;
    }
    GroupMemoryBarrierWithGroupSync();

    uint counts_and_indices[STEPS * STEPS * STEPS];
    uint nonzeros = 0;

    uint base_color_code = gid.z << 19 | gid.y << 14 | gid.x << 9;
    for (uint i = 0; i < STEPS * STEPS * STEPS; ++i) {
        uint index = i << 6 | tig;
        uint color_code = base_color_code | index;
        uint count = CountBuf[color_code];

        if (count > 0) {
            counts_and_indices[nonzeros] = min(count, 0x7FFFFF) << 9 | index;
            ++nonzeros;
        }
    }

    uint nonzeros_prefix_in_wave = WavePrefixSum(nonzeros);
    uint nonzeros_in_wave = WaveActiveSum(nonzeros);

    uint nonzeros_prefix_in_group;
    if (WaveIsFirstLane()) {
        InterlockedAdd(nonzeros_in_group, nonzeros_in_wave, nonzeros_prefix_in_group);
    }
    nonzeros_prefix_in_group = WaveReadLaneFirst(nonzeros_prefix_in_group);

    uint offset = nonzeros_prefix_in_group + nonzeros_prefix_in_wave;
    for (uint i = 0; i < nonzeros; ++i) {
        payload.counts_and_indices[offset + i] = counts_and_indices[i];
    }

    if (tig == 0) {
        payload.num = nonzeros_in_group;
        payload.base_color_code = base_color_code;
    }
    GroupMemoryBarrierWithGroupSync();

    uint num_dispatch = (nonzeros_in_group + ELEMS - 1) / ELEMS;
    DispatchMesh(num_dispatch, 1, 1, payload);
}

float3 RgbToPosition(float3 rgb) {
    return 1.25 * (rgb - 0.5);
}

float3 RgbToHslPosition(float3 rgb) {
    float3 hsl = RgbToHsl(rgb);

    float h = 2.0 * PI * hsl.x; // 0 - 2pi
    float s = hsl.y; // 0 - 1
    float l = 2.0 * hsl.z - 1.0; // -1 - 1

    float a = s + abs(l);
    float b = sqrt(s * s + l * l);
    if (b > 0) {
        float n = a / b;
        s *= n;
        l *= n;
    }
    
    float y = l;
    float x, z;
    sincos(h, z, x);

    x *= s;
    z *= s;

    return float3(x, y, -z);
}

struct VertexOut {
    float4 position : SV_Position;
    float4 color : COLOR;
    float2 uv : TEXCOORD;
};

VertexOut GetVertexAttribute(float3 center, float3 color, float scale, float2 uv) {
    float3 position = center;
    position.xy += scale * uv;

    VertexOut vert;
    vert.position = float4(position, 1.0);
    vert.color = float4(color, 1.0);
    vert.uv = uv;

    return vert;
}

#define PRIMITIVES (1 * ELEMS)
#define VERTICES (3 * ELEMS)
[outputtopology("triangle")]
[numthreads(ELEMS, 1, 1)]
void ColorCloudMs(uint id: SV_DispatchThreadID, uint gid: SV_GroupID, uint tid: SV_GroupThreadID, in payload Payload payload, out vertices VertexOut vertes[VERTICES], out indices uint3 tris[PRIMITIVES]) {
    uint num_elems = min(payload.num - (ELEMS * gid), ELEMS);
    SetMeshOutputCounts(3 * num_elems, 1 * num_elems);

    if (id < payload.num ) {
        uint count_and_index = payload.counts_and_indices[id];
        uint count = count_and_index >> 9;
        uint index = 0x01FF & count_and_index;
        uint color_code = payload.base_color_code | index;
        float3 color = IntToRgb(color_code);
        float rate = saturate(float(count - MinCount) * InvMaxCount);
        float scale = lerp(0.0005, 0.1, sqrt(rate));

        float3 center;
        switch (ColorSpace) {
        case 0: center = RgbToPosition(color); break;
        case 1: center = RgbToHslPosition(color); break;
        }
        
        center = mul(float4(center, 1.0), Projection);

        uint vindex = 3 * tid;
        vertes[vindex + 0] = GetVertexAttribute(center, color, scale, float2(-1.0, +3.0));
        vertes[vindex + 1] = GetVertexAttribute(center, color, scale, float2(+3.0, -1.0));
        vertes[vindex + 2] = GetVertexAttribute(center, color, scale, float2(-1.0, -1.0));

        uint pindex = tid;
        tris[pindex] = uint3(vindex + 0, vindex + 1, vindex + 2);
    }
}

struct PsInput {
    float4 position : SV_Position;
    float4 color : COLOR;
    float2 uv : TEXCOORD;
};

float4 ColorCloudPs(PsInput input) : SV_Target {
    clip(1.0 - dot(input.uv, input.uv));
    return input.color;
}

#endif // GRAPHICS
