use threshold::ThresholdEffect;
use windows::{core::Result, Win32::Graphics::Direct2D::ID2D1Factory1};

macro_rules! create_setter_helpers {
    ($impl_wraper:ty, $setter_method:ident, $getter_method:ident, $value_ty:ty, $helper_mod:ident) => {
        mod $helper_mod {
            use super::*;
            use windows::{
                core::{IUnknown, Result},
                Win32::Foundation::E_INVALIDARG,
            };
            unsafe fn value_setter_impl(
                effect: IUnknown,
                data: *const u8,
                data_size: u32,
            ) -> Result<()> {
                let effect_ptr = std::mem::transmute::<_, *mut $impl_wraper>(effect);
                let effect = effect_ptr.as_mut().unwrap();

                if data_size != std::mem::size_of::<$value_ty>() as u32 {
                    return E_INVALIDARG.ok();
                }

                let value = *(data as *const $value_ty);
                effect.this.$setter_method(value)?;

                Ok(())
            }
            pub unsafe extern "system" fn value_setter(
                effect: Option<IUnknown>,
                data: *const u8,
                data_size: u32,
            ) -> HRESULT {
                if let Some(effect) = effect {
                    if let Err(error) = value_setter_impl(effect, data, data_size) {
                        error.code()
                    } else {
                        S_OK
                    }
                } else {
                    E_INVALIDARG
                }
            }

            unsafe fn value_getter_impl(
                effect: IUnknown,
                data: *mut u8,
                data_size: u32,
                actual_size: &mut u32,
            ) -> Result<()> {
                let effect_ptr = std::mem::transmute::<_, *mut $impl_wraper>(effect);
                let effect = effect_ptr.as_mut().unwrap();

                let value_size = std::mem::size_of::<$value_ty>() as u32;
                if data_size < value_size {
                    return E_INVALIDARG.ok();
                }
                *actual_size = value_size;

                let value_ptr = data as *mut $value_ty;
                if let Some(value) = value_ptr.as_mut() {
                    *value = effect.this.$getter_method();
                    Ok(())
                } else {
                    E_INVALIDARG.ok()
                }
            }
            pub unsafe extern "system" fn value_getter(
                effect: Option<IUnknown>,
                data: *mut u8,
                data_size: u32,
                actual_size: *mut u32,
            ) -> HRESULT {
                if let Some(effect) = effect {
                    if let Some(actual_size) = actual_size.as_mut() {
                        if let Err(error) = value_getter_impl(effect, data, data_size, actual_size)
                        {
                            error.code()
                        } else {
                            S_OK
                        }
                    } else {
                        E_INVALIDARG
                    }
                } else {
                    E_INVALIDARG
                }
            }
        }
    };
}

pub mod threshold;

pub fn register_custom_effects(d2d_factory: &ID2D1Factory1) -> Result<()> {
    ThresholdEffect::register(&d2d_factory)?;
    Ok(())
}