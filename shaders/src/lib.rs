pub fn threshold_shader() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/shaders/ThresholdShader.cso"))
}
