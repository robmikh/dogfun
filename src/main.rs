mod d2d;
mod d3d11;

use d2d::{create_d2d_device, create_d2d_factory};
use d3d11::{create_d3d_device, create_direct3d_surface};
use windows::{
    core::{h, Interface, Result, HSTRING},
    Graphics::Imaging::{
        BitmapAlphaMode, BitmapBufferAccessMode, BitmapDecoder, BitmapEncoder, BitmapPixelFormat,
        SoftwareBitmap,
    },
    Storage::{
        CreationCollisionOption, FileAccessMode, StorageFolder,
        Streams::{
            IRandomAccessStream, InMemoryRandomAccessStream, InputStreamOptions, RandomAccessStream,
        },
    },
    Win32::{
        Graphics::{
            Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
            Direct3D11::{
                D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
                D3D11_USAGE_DEFAULT,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGISurface,
            },
        },
        System::WinRT::{
            CreateRandomAccessStreamOnFile, IMemoryBufferByteAccess, RoInitialize,
            RO_INIT_MULTITHREADED,
        },
    },
    UI::Composition::CompositionSurfaceBrush,
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let image_path = args.get(0).expect("Expected input file path!");

    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }

    // Init D3D11 and D2D
    let d3d_device = create_d3d_device()?;
    let d2d_factory = create_d2d_factory()?;
    let d2d_device = create_d2d_device(&d2d_factory, &d3d_device)?;
    let d2d_context = unsafe { d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? };

    // Load and decode the input image
    let stream: IRandomAccessStream = unsafe {
        CreateRandomAccessStreamOnFile(&HSTRING::from(image_path), FileAccessMode::Read.0 as u32)?
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

    // Get bitmap dimensions
    let width = converted_bitmap.PixelWidth()? as u32;
    let height = converted_bitmap.PixelHeight()? as u32;

    // Get the raw bytes
    let bytes = {
        let bitmap_buffer = converted_bitmap.LockBuffer(BitmapBufferAccessMode::Read)?;
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

    // Create our input texture
    let input_texture = {
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

    // Create our input bitmap
    let input_bitmap = {
        let surface: IDXGISurface = input_texture.cast()?;
        unsafe { d2d_context.CreateBitmapFromDxgiSurface(&surface, None)? }
    };

    // DEBUG: Use the input texture as our output texture
    let output_texture = input_texture.clone();

    // Save the output
    let output_surface = create_direct3d_surface(&output_texture)?;
    let output_software_bitmap = SoftwareBitmap::CreateCopyWithAlphaFromSurfaceAsync(
        &output_surface,
        BitmapAlphaMode::Premultiplied,
    )?
    .get()?;
    let output_file = {
        let current_dir = std::env::current_dir()?;
        let folder =
            StorageFolder::GetFolderFromPathAsync(&HSTRING::from(current_dir.as_path()))?.get()?;
        let file = folder
            .CreateFileAsync(h!("dog.png"), CreationCollisionOption::ReplaceExisting)?
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

    println!("Done!");

    Ok(())
}
