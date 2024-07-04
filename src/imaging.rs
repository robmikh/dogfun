use std::path::Path;

use windows::{
    core::{Interface, Result, HSTRING},
    Graphics::Imaging::{
        BitmapAlphaMode, BitmapBuffer, BitmapBufferAccessMode, BitmapDecoder, BitmapEncoder,
        BitmapPixelFormat, SoftwareBitmap,
    },
    Storage::{
        CreationCollisionOption, FileAccessMode, StorageFolder, Streams::IRandomAccessStream,
    },
    Win32::{
        Graphics::{
            Direct3D11::{
                ID3D11Device, ID3D11Texture2D, D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA,
                D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
            },
            Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        },
        System::WinRT::{CreateRandomAccessStreamOnFile, IMemoryBufferByteAccess},
    },
};

use crate::d3d11::create_direct3d_surface;

pub fn load_bitmap_from_path<P: AsRef<Path>>(path: P) -> Result<SoftwareBitmap> {
    let path = path.as_ref();

    // Get the SoftwareBitmap
    let stream: IRandomAccessStream = unsafe {
        CreateRandomAccessStreamOnFile(&HSTRING::from(path), FileAccessMode::Read.0 as u32)?
    };
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
    let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

    // Convert to premulitplied alpha if necessary
    let converted_bitmap = if software_bitmap.BitmapPixelFormat()? != BitmapPixelFormat::Bgra8
        || software_bitmap.BitmapAlphaMode()? != BitmapAlphaMode::Premultiplied
    {
        SoftwareBitmap::ConvertWithAlpha(
            &software_bitmap,
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
        )?
    } else {
        software_bitmap
    };

    Ok(converted_bitmap)
}

pub fn get_bytes_from_bitmap<'a>(bitmap_buffer: &'a BitmapBuffer) -> Result<&'a [u8]> {
    let bytes = {
        let reference = bitmap_buffer.CreateReference()?;
        let byte_access: IMemoryBufferByteAccess = reference.cast()?;

        let mut bytes_ptr = std::ptr::null_mut();
        let mut len = 0;
        unsafe {
            byte_access.GetBuffer(&mut bytes_ptr, &mut len)?;
        }

        let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, len as usize) };
        bytes
    };

    Ok(bytes)
}

pub fn create_texture_from_bitmap(
    d3d_device: &ID3D11Device,
    software_bitmap: &SoftwareBitmap,
) -> Result<ID3D11Texture2D> {
    // Get bitmap dimensions
    let width = software_bitmap.PixelWidth()? as u32;
    let height = software_bitmap.PixelHeight()? as u32;

    // Get the raw bytes
    let bitmap_buffer = software_bitmap.LockBuffer(BitmapBufferAccessMode::Read)?;
    let bytes = get_bytes_from_bitmap(&bitmap_buffer)?;

    // Create our input texture
    let texture = {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
            ..Default::default()
        };

        let subresource_init = D3D11_SUBRESOURCE_DATA {
            pSysMem: bytes.as_ptr() as *const _,
            SysMemPitch: width * 4, // 4 bytes per pixel (BGRA8)
            ..Default::default()
        };

        unsafe {
            let mut texture = None;
            d3d_device.CreateTexture2D(&desc, Some(&subresource_init), Some(&mut texture))?;
            texture.unwrap()
        }
    };

    Ok(texture)
}

pub fn save_texture_to_path<P: AsRef<Path>>(texture: &ID3D11Texture2D, path: P) -> Result<()> {
    let path = path.as_ref();

    let output_surface = create_direct3d_surface(&texture)?;
    let output_software_bitmap = SoftwareBitmap::CreateCopyWithAlphaFromSurfaceAsync(
        &output_surface,
        BitmapAlphaMode::Premultiplied,
    )?
    .get()?;
    let output_file = {
        // TODO: Don't use current dir
        let current_dir = std::env::current_dir()?;
        let folder =
            StorageFolder::GetFolderFromPathAsync(&HSTRING::from(current_dir.as_path()))?.get()?;
        let file = folder
            .CreateFileAsync(
                &HSTRING::from(path),
                CreationCollisionOption::ReplaceExisting,
            )?
            .get()?;
        file
    };
    {
        let output_stream = output_file.OpenAsync(FileAccessMode::ReadWrite)?.get()?;
        let encoder =
            BitmapEncoder::CreateAsync(BitmapEncoder::PngEncoderId()?, &output_stream)?.get()?;
        encoder.SetSoftwareBitmap(&output_software_bitmap)?;
        encoder.FlushAsync()?.get()?;
    }

    Ok(())
}
