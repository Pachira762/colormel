#include "common.hlsli"

#ifdef GRAPHICS

cbuffer Params : register(b0) {
    float4x3 Projection;
};

struct VertexOut {
    float4 position : SV_Position;
    float4 color : COLOR;
};

VertexOut PrimitiveVs(float3 position: POSITION, float3 color: COLOR) {
    VertexOut output;

    output.position = float4(mul(float4(position, 1.0), Projection), 1.0);
    output.color = float4(color, 1.0);

    return output;
}

float4 PrimitivePs(float4 position: SV_Position, float4 color: COLOR) : SV_Target {
    return color;
}

#endif // GRAPHICS
