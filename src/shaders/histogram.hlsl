#include "common.hlsli"

#ifdef COMPUTE

cbuffer Params : register(b0) {
    int4 Rect;
    uint Mode;
    uint Ch;
};

#define MAX_CH 4
RWBuffer<uint> HistogramBuf[MAX_CH] : register(u0);

#define N_BINS 256
groupshared uint bins[MAX_CH][N_BINS];

#define SCALE 2
#define THREAD_X 8
#define THREADS (THREAD_X * THREAD_X)
[numthreads(THREAD_X, THREAD_X, 1)]
void HistogramCs(uint2 id: SV_DispatchThreadID, uint2 gid: SV_GroupID, uint tig: SV_GroupIndex) {
    for (uint i = tig; i < N_BINS; i += THREADS) {
        for (uint ch = 0; ch < Ch; ++ch) {
            bins[ch][i] = 0;
        }
    }
    
    GroupMemoryBarrierWithGroupSync();

    uint2 pixpos0 = Rect.xy + SCALE * id;
    uint ibins[MAX_CH] = {0, 0, 0, 0};
    uint counts[MAX_CH] = {1, 1, 1, 1};

    for (uint y = 0; y < SCALE; ++y) {
        for (uint x = 0; x < SCALE; ++x) {
            uint2 pixpos = pixpos0 + uint2(x, y);
            if (all(pixpos < Rect.zw)) {
                float3 color = Desktop[pixpos].rgb;

                switch (Mode) {
                    case 0: // RGB
                        ibins[0] = (N_BINS - 1) * color.g;
                        ibins[1] = (N_BINS - 1) * color.r;
                        ibins[2] = (N_BINS - 1) * color.b;
                        break;

                    case 1: // RGBL
                        ibins[0] = (N_BINS - 1) * color.g;
                        ibins[1] = (N_BINS - 1) * color.r;
                        ibins[2] = (N_BINS - 1) * color.b;
                        ibins[3] = (N_BINS - 1) * Luma(color);
                        break;

                    case 2: // Luma
                        ibins[0] = (N_BINS - 1) * Luma(color);
                        break;

                    case 3: { // Hue 
                        float3 hsl = RgbToHsl(color);
                        ibins[0] = (N_BINS - 1) * hsl.x;
                        counts[0] = hsl.y > 0 ? (127.0 * (0.5 * hsl.y + 0.5)) : 0;
                        break;
                    }
                }

                for (uint ch = 0; ch < Ch; ++ch) {
                    uint i = ibins[ch];
                    InterlockedAdd(bins[ch][i], counts[ch]);
                }
            }
        }
    }
    
    GroupMemoryBarrierWithGroupSync();
    
    for(uint i = tig; i < N_BINS; i += THREADS) {
        for (uint ch = 0; ch < Ch; ++ch) {
            uint count = bins[ch][i];
            if (count > 0) {
                InterlockedAdd(HistogramBuf[ch][i], count);
            }
        }
    }
}

#endif // COMPUTE

#ifdef GRAPHICS

#define MAX_CH 4

cbuffer Params : register(b0) {
    float4 Colors[MAX_CH];
    uint Mode;
    float Scale;
};

Buffer<uint> HistogramBuf[MAX_CH] : register(t0);

struct VertexOut {
    float4 position : SV_Position;
    float4 color : COLOR;
};

VertexOut HistogramVs(uint vid: SV_VertexID, uint iid: SV_InstanceID) {
    uint index = vid / 2;
    uint count = HistogramBuf[iid][index];
    bool bottom = vid % 2 == 0;

    float x = 2.f * (float(index) / 255.f) - 1.f;
    float y = bottom ? -1.f : (Scale * count - 1.f);

    VertexOut output;
    output.position = float4(x, y, 0.f, 1.f);

    switch (Mode) {
        case 0: // RGB
            output.color = Colors[iid];
            break;

        case 1: // RGBL
            output.color = Colors[iid];
            break;

        case 2: // Luma
            output.color = Colors[3];
            break;

        case 3: { // Hue 
            float3 rgb = HslToRgb((float)index / 256.f, 1.0f, 0.5f);
            output.color = float4(rgb, Colors[3].a);
            break;
        }
    }

    return output;
}

float4 HistogramPs(VertexOut input) : SV_Target {
    float4 color = input.color;
    color.rgb *= color.a;
    return color;
}

#endif // GRAPHICS
