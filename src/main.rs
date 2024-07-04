mod d2d;
mod d3d11;
mod imaging;

use d2d::{create_d2d_device, create_d2d_factory};
use d3d11::create_d3d_device;
use imaging::{create_texture_from_bitmap, load_bitmap_from_path, save_texture_to_path};
use windows::{
    core::{Interface, Result},
    Win32::{
        Graphics::{
            Direct2D::{
                CLSID_D2D1Blend, CLSID_D2D1GaussianBlur,
                Common::{
                    D2D1_BLEND_MODE_SUBTRACT, D2D1_BORDER_MODE_HARD,
                    D2D1_COMPOSITE_MODE_SOURCE_OVER,
                },
                ID2D1Bitmap, ID2D1DeviceContext, ID2D1Effect, ID2D1Image, D2D1_BLEND_PROP_MODE,
                D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_GAUSSIANBLUR_PROP_BORDER_MODE,
                D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION, D2D1_INTERPOLATION_MODE_LINEAR,
                D2D1_PROPERTY_TYPE_FLOAT, D2D1_PROPERTY_TYPE_UNKNOWN,
            },
            Direct3D11::{
                D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_TEXTURE2D_DESC,
            },
            Dxgi::IDXGISurface,
        },
        System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
    },
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
    let software_bitmap = load_bitmap_from_path(&image_path)?;

    // Create our input texture
    let input_texture = create_texture_from_bitmap(&d3d_device, &software_bitmap)?;

    // Create our input bitmap
    let input_bitmap = {
        let surface: IDXGISurface = input_texture.cast()?;
        unsafe { d2d_context.CreateBitmapFromDxgiSurface(&surface, None)? }
    };

    // Create our output texture
    let output_texture = {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            input_texture.GetDesc(&mut desc);
        }
        desc.BindFlags = (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0) as u32;

        unsafe {
            let mut texture = None;
            d3d_device.CreateTexture2D(&desc, None, Some(&mut texture))?;
            texture.unwrap()
        }
    };

    // Create our output bitmap
    let output_bitmap = {
        let surface: IDXGISurface = output_texture.cast()?;
        unsafe { d2d_context.CreateBitmapFromDxgiSurface(&surface, None)? }
    };

    // Setup our effect graph
    unsafe {
        d2d_context.SetTarget(&output_bitmap);
    }
    let blur_1 = create_gaussian_blur(&d2d_context, &input_bitmap, 3.0)?;
    let blur_1_image: ID2D1Image = blur_1.cast()?;
    let blur_2 = create_gaussian_blur(&d2d_context, &input_bitmap, 5.0)?;
    let blur_2_image: ID2D1Image = blur_2.cast()?;
    let subtract_effect = create_subtract_effect(&d2d_context, &blur_1_image, &blur_2_image)?;
    let subtract_image: ID2D1Image = subtract_effect.cast()?;

    // Draw
    unsafe {
        d2d_context.BeginDraw();
        d2d_context.Clear(None);
        d2d_context.DrawImage(
            &subtract_image,
            None,
            None,
            D2D1_INTERPOLATION_MODE_LINEAR,
            D2D1_COMPOSITE_MODE_SOURCE_OVER,
        );
        d2d_context.EndDraw(None, None)?;
    }

    // Save the output
    save_texture_to_path(&output_texture, "dog.png")?;

    println!("Done!");

    Ok(())
}

fn create_gaussian_blur(
    d2d_context: &ID2D1DeviceContext,
    input: &ID2D1Bitmap,
    standard_deviation: f32,
) -> Result<ID2D1Effect> {
    let effect = unsafe { d2d_context.CreateEffect(&CLSID_D2D1GaussianBlur)? };

    unsafe {
        effect.SetInput(0, input, None);
        let value = standard_deviation.to_le_bytes();
        effect.SetValue(
            D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0 as u32,
            D2D1_PROPERTY_TYPE_FLOAT,
            &value,
        )?;
        let value = D2D1_BORDER_MODE_HARD.0.to_le_bytes();
        effect.SetValue(
            D2D1_GAUSSIANBLUR_PROP_BORDER_MODE.0 as u32,
            D2D1_PROPERTY_TYPE_UNKNOWN,
            &value,
        )?;
    }

    Ok(effect)
}

fn create_subtract_effect(
    d2d_context: &ID2D1DeviceContext,
    input_1: &ID2D1Image,
    input_2: &ID2D1Image,
) -> Result<ID2D1Effect> {
    let effect = unsafe { d2d_context.CreateEffect(&CLSID_D2D1Blend)? };

    unsafe {
        effect.SetInput(0, input_1, None);
        effect.SetInput(1, input_2, None);
        let value = D2D1_BLEND_MODE_SUBTRACT.0.to_le_bytes();
        effect.SetValue(
            D2D1_BLEND_PROP_MODE.0 as u32,
            D2D1_PROPERTY_TYPE_UNKNOWN,
            &value,
        )?;
    }

    Ok(effect)
}
