use windows::{
    core::Result,
    Win32::Graphics::Direct3D11::{
        ID3D11Buffer, ID3D11ComputeShader, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
        ID3D11UnorderedAccessView, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE,
        D3D11_BIND_UNORDERED_ACCESS, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE,
        D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE, D3D11_TEX2D_UAV, D3D11_TEXTURE2D_DESC,
        D3D11_UAV_DIMENSION_TEXTURE2D, D3D11_UNORDERED_ACCESS_VIEW_DESC,
        D3D11_UNORDERED_ACCESS_VIEW_DESC_0, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
    },
};

pub struct ThresholdEffect {
    params_buffer: ID3D11Buffer,
    params_staging_buffer: ID3D11Buffer,
    shader: ID3D11ComputeShader,
}

#[repr(C, align(16))]
struct Parameters {
    threshold_value: f32,
}

impl ThresholdEffect {
    pub fn new(d3d_device: &ID3D11Device) -> Result<Self> {
        let (params_buffer, params_staging_buffer) = unsafe {
            let mut desc = D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<Parameters>() as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
                ..Default::default()
            };
            let input_info_buffer = {
                let mut buffer = None;
                d3d_device.CreateBuffer(&desc, None, Some(&mut buffer))?;
                buffer.unwrap()
            };

            desc.Usage = D3D11_USAGE_STAGING;
            desc.BindFlags = 0;
            desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE.0 as u32;
            let input_info_staging_buffer = {
                let mut buffer = None;
                d3d_device.CreateBuffer(&desc, None, Some(&mut buffer))?;
                buffer.unwrap()
            };

            (input_info_buffer, input_info_staging_buffer)
        };

        let shader = unsafe {
            let mut shader = None;
            d3d_device.CreateComputeShader(shaders::threshold_shader(), None, Some(&mut shader))?;
            shader.unwrap()
        };

        Ok(Self {
            params_buffer,
            params_staging_buffer,
            shader,
        })
    }

    pub fn run(
        &self,
        d3d_device: &ID3D11Device,
        d3d_context: &ID3D11DeviceContext,
        input: &ID3D11Texture2D,
        threshold: f32,
    ) -> Result<ID3D11Texture2D> {
        let desc = unsafe {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            input.GetDesc(&mut desc);
            desc
        };

        let input_srv = unsafe {
            let mut srv = None;
            d3d_device.CreateShaderResourceView(input, None, Some(&mut srv))?;
            srv.unwrap()
        };

        let (output_texture, output_uav) = self.create_output_texture(d3d_device, &desc)?;

        // Update info buffer
        unsafe {
            let info = Parameters {
                threshold_value: threshold,
            };
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            d3d_context.Map(
                &self.params_staging_buffer,
                0,
                D3D11_MAP_WRITE,
                0,
                Some(&mut mapped),
            )?;

            let info_staging = mapped.pData as *mut Parameters;
            (*info_staging) = info;

            d3d_context.Unmap(&self.params_staging_buffer, 0);
            d3d_context.CopyResource(&self.params_buffer, &self.params_staging_buffer);
        }

        // Run the shader
        unsafe {
            d3d_context.CSSetShader(&self.shader, None);
            d3d_context.CSSetShaderResources(0, Some(&[Some(input_srv.clone())]));
            d3d_context.CSSetConstantBuffers(0, Some(&[Some(self.params_buffer.clone())]));
            d3d_context.CSSetUnorderedAccessViews(
                0,
                1,
                Some(&[Some(output_uav.clone())] as *const _),
                None,
            );

            let (thread_x_count, thread_y_count) =
                compute_xy_thread_group_count(desc.Width, desc.Height, 16);
            d3d_context.Dispatch(thread_x_count, thread_y_count, 1);
            d3d_context.CSSetShaderResources(0, Some(&[None]));
            d3d_context.CSSetConstantBuffers(0, Some(&[None]));
            d3d_context.CSSetUnorderedAccessViews(0, 1, Some(&[None] as *const _), None);
        }

        Ok(output_texture)
    }

    fn create_output_texture(
        &self,
        d3d_device: &ID3D11Device,
        desc: &D3D11_TEXTURE2D_DESC,
    ) -> Result<(ID3D11Texture2D, ID3D11UnorderedAccessView)> {
        let mut desc = *desc;
        desc.BindFlags = (D3D11_BIND_UNORDERED_ACCESS.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32;
        desc.MiscFlags = 0;

        let mut texture = None;
        unsafe {
            d3d_device.CreateTexture2D(&desc, None, Some(&mut texture))?;
        }
        let texture = texture.unwrap();

        let uav = unsafe {
            let desc = D3D11_UNORDERED_ACCESS_VIEW_DESC {
                Format: desc.Format,
                ViewDimension: D3D11_UAV_DIMENSION_TEXTURE2D,
                Anonymous: D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Texture2D: D3D11_TEX2D_UAV { MipSlice: 0 },
                },
                ..Default::default()
            };
            let uav = {
                let mut uav = None;
                d3d_device.CreateUnorderedAccessView(&texture, Some(&desc), Some(&mut uav))?;
                uav.unwrap()
            };
            uav
        };

        Ok((texture, uav))
    }
}

fn compute_xy_thread_group_count(
    width: u32,
    height: u32,
    num_threads_per_dimension: u32,
) -> (u32, u32) {
    (
        compute_thread_group_count(width, num_threads_per_dimension),
        compute_thread_group_count(height, num_threads_per_dimension),
    )
}

fn compute_thread_group_count(value: u32, threads: u32) -> u32 {
    let base = value / threads;
    if value % threads == 0 {
        base
    } else {
        base + 1
    }
}
