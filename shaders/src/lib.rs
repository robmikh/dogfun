pub fn threshold_pixel_shader() -> &'static [u8] {
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/shaders/ThresholdPixelShader.cso"
    ))
}
