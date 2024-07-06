#define D2D_INPUT_COUNT 1 
#define D2D_INPUT0_SIMPLE

#include "d2d1effecthelpers.hlsli"

cbuffer constants : register(b0)
{
    float ThresholdValue : packoffset(c0.x);
};

float ApplyThreshold(float value)
{
    if (value >= ThresholdValue)
    {
        return 1;
    }
    return 0;
}

D2D_PS_ENTRY(main)
{
    float4 pixel = D2DGetInput(0);

    pixel.x = ApplyThreshold(pixel.x);
    pixel.y = ApplyThreshold(pixel.y);
    pixel.z = ApplyThreshold(pixel.z);
    pixel.w = 1.0f;

    return pixel;
}