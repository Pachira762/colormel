#include "common.hlsli"

#ifdef GRAPHICS

cbuffer Params : register(b0) {
    int4 Rect;
    int Mode;
    float3 ColorMask;
}

#define FILTER_MODE_RGB 0
#define FILTER_MODE_HUE 1
#define FILTER_MODE_SAT 2
#define FILTER_MODE_LUMA 3

struct PsInput {
    float4 position: SV_Position;
};

PsInput FilterVs(uint id: SV_VertexID) {
    static const float2 Positions[3] = {
        float2(-1, +3),
        float2(+3, -1),
        float2(-1, -1),
    };
    
    PsInput output;
    output.position = float4(Positions[id], 0.0, 1.0);

    return output;
}

float3 remap(float3 v, float v_min, float v_max) {
    return (v - v_min) / (v_max - v_min);
}

float4 FilterPs(float4 position: SV_Position) : SV_Target {
    int2 pixpos = Rect.xy + int2(position.xy);
    float3 rgb = Desktop[pixpos].rgb;

    float3 out_color;

    switch (Mode) {
        case FILTER_MODE_RGB: {
            out_color = ColorMask * rgb;

            break;
        }

        case FILTER_MODE_HUE: {
            float mi = Min3(rgb.r, rgb.g, rgb.b);
            float ma = Max3(rgb.r, rgb.g, rgb.b);
            float d = ma - mi;

            if (d > 0.0) {
                out_color = remap(rgb, mi, ma);
            } else {
                out_color = rgb;
            }

            break;
        }

        case FILTER_MODE_SAT: {
            float mi = Min3(rgb.r, rgb.g, rgb.b);
            float ma = Max3(rgb.r, rgb.g, rgb.b);
            float l = 0.5 * (mi + ma);

            if (l == 0.0 || l == 1.0) {
                out_color = (float3) 0;
            }else {
                float s = (ma - mi) / (1.0 - abs(ma + mi - 1.0));
                out_color = float3(s, s, s);
            }

            break;
        }

        case FILTER_MODE_LUMA: {
            float l = Luma(rgb);
            out_color = float3(l, l, l);

            break;
        }
    }

    return float4(out_color, 1.0);
}

#endif // GRAPHICS
