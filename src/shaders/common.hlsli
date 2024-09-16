Texture2D<float4> Desktop : register(t0, space1);

static const float PI = 3.14159265358979323846;

float Max3(float a, float b, float c)  {
    return max(a, max(b, c));
}

float Min3(float a, float b, float c) {
    return min(a, min(b, c));
}

float Luma(float3 rgb) {
    static const float3 LUMA = {
        0.2126, 0.7152, 0.0722f
    };

    return dot(rgb, LUMA);
}

float3 RgbToHsv(float3 rgb) {
    float ma = Max3(rgb.r, rgb.g, rgb.b);
    float mi = Min3(rgb.r, rgb.g, rgb.b);

    float H = 0.0;
    if (mi == ma) {
        H = 0.0;
    }
    else if (mi == rgb.b) {
        H = ((rgb.g - rgb.r) / (ma - mi) + 1.0) / 6.0;
    }
    else if (mi == rgb.r) {
        H = ((rgb.b - rgb.g) / (ma - mi) + 3.0) / 6.0;
    }
    else {
        H = ((rgb.r - rgb.b) / (ma - mi) + 5.0) / 6.0;
    }

    float S = ma - mi;

    float V = ma;

    return float3(H, S, V);
}

float3 RgbToHsl(float3 rgb) {
    float ma = Max3(rgb.r, rgb.g, rgb.b);
    float mi = Min3(rgb.r, rgb.g, rgb.b);

    float H = 0.0;
    if (mi == ma) {
        H = 0.0;
    }
    else if (mi == rgb.b) {
        H = ((rgb.g - rgb.r) / (ma - mi) + 1.0) / 6.0;
    }
    else if (mi == rgb.r) {
        H = ((rgb.b - rgb.g) / (ma - mi) + 3.0) / 6.0;
    }
    else {
        H = ((rgb.r - rgb.b) / (ma - mi) + 5.0) / 6.0;
    }

    float S = ma - mi;

    float L = (ma + mi) / 2.0;

    return float3(H, S, L);
}

float3 RgbToXyz(float3 rgb) {
    static const float3x3 RGB_TO_XYZ = {
        0.412391, 0.357584, 0.180481,
        0.212639, 0.715169, 0.072192,
        0.019331, 0.119195, 0.950532,
    };

    return mul(RGB_TO_XYZ, rgb);
}

float3 RgbToYuv(float3 rgb) {
    static const float3x3 RgbToYuv = {
        +0.212600, +0.715200, +0.072200,
        -0.114572, -0.385428, +0.500000,
        +0.500000, -0.451453, -0.045847
    };

    return mul(RgbToYuv, rgb) + float3(0.0, 0.5, 0.5);
}

float3 HsvToRgb(float hue, float saturation, float luminance) {
    float r = luminance;
    float g = luminance;
    float b = luminance;

    float h = frac(hue + 1.0);
    uint i = (uint)(6.0 * h);
    float f = 6.0 * h - (float)i;
    float s = saturation;

    switch (i) {
    case 0:
        g *= 1.0 - s * (1.0 - f);
        b *= 1.0 - s;
        break;
    case 1:
        r *= 1.0 - s * f;
        b *= 1.0 - s;
        break;
    case 2:
        r *= 1.0 - s;
        b *= 1.0 - s * (1.0 - f);
        break;
    case 3:
        r *= 1.0 - s;
        g *= 1.0 - s * f;
        break;
    case 4:
        r *= 1.0 - s * (1.0 - f);
        g *= 1.0 - s;
        break;
    case 5:
        g *= 1.0 - s;
        b *= 1.0 - s * f;
        break;
    }

    return float3(r, g, b);
}

float3 HslToRgb(float hue, float saturation, float luminance) {
    float h = 360.0 * frac(hue + 1.0);
    float ma = luminance + 0.5 * saturation;
    float mi = luminance - 0.5 * saturation;
    float mm = ma - mi;

    if (h < 60.0) {
        return float3(ma, mi + mm * h / 60.0, mi);
    } else if (h < 120.0) {
        return float3(mi + mm * (120.0 - h) / 60.0, ma, mi);
    } else if (h < 180.0) {
        return float3(mi, ma, mi + mm * (h - 120.0) / 60.0);
    } else if (h < 240.0) {
        return float3(mi, mi + mm * (240.0 - h)/60.0, ma);
    } else if (h < 300.0) {
        return float3(mi + mm * (h-240.0) / 60.0, mi, ma);
    } else {
        return float3(ma, mi, mi + mm*(360.0 - h) / 60.0);
    }
}

uint RgbToInt(float3 rgb) {
    uint3 color = uint3(255.0 * rgb);
    return color.r | color.g << 8 | color.b << 16;
}

float3 IntToRgb(uint color) {
    return float3(color & 0xff, (color & 0xff00) >> 8, (color & 0xff0000) >> 16) / 255.0;
}
