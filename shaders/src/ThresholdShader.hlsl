cbuffer Parameters : register(b0)
{
    float ThresholdValue;
};

Texture2D<unorm float4> inputTexture : register(t0);

RWTexture2D<unorm float4> outputTexture : register(u0);

float ApplyThreshold(float value)
{
    if (value >= ThresholdValue)
    {
        return 1;
    }
    return 0;
}

[numthreads(16, 16, 1)]
void main( uint3 DTid : SV_DispatchThreadID )
{
    uint2 position = DTid.xy;

    uint width;
    uint height;
    inputTexture.GetDimensions(width, height);

    if (position.x < width && position.y < height)
    {
        float4 pixel = inputTexture[position];

        pixel.x = ApplyThreshold(pixel.x);
        pixel.y = ApplyThreshold(pixel.y);
        pixel.z = ApplyThreshold(pixel.z);
        pixel.w = 1.0f;

        outputTexture[position] = pixel;
    }
}