use windows::{
    core::{implement, interface, w, IUnknown, Result, GUID, HRESULT, HSTRING, PCWSTR},
    Win32::{
        Foundation::{E_INVALIDARG, RECT, S_OK},
        Graphics::Direct2D::{
            ID2D1DrawInfo, ID2D1DrawTransform, ID2D1DrawTransform_Impl, ID2D1EffectContext,
            ID2D1EffectImpl, ID2D1EffectImpl_Impl, ID2D1Factory1, ID2D1TransformGraph,
            ID2D1TransformNode, ID2D1TransformNode_Impl, ID2D1Transform_Impl, D2D1_CHANGE_TYPE,
            D2D1_PROPERTY_BINDING,
        },
    },
};
use windows_core::Interface;

pub const CLSID_SampleEffect: GUID = GUID::from_u128(0xFB3AF5AA_6F03_4754_A676_BACB2D082069);
pub const SAMPLE_EFFECT_SHADER: GUID = GUID::from_u128(0x397DBC73_8831_4C02_9ECD_56036DAEA108);

#[implement(ID2D1EffectImpl, ID2D1DrawTransform)]
pub struct SampleEffect {
    threshold: f32,
}

impl ID2D1EffectImpl_Impl for SampleEffect_Impl {
    fn Initialize(
        &self,
        effectcontext: Option<&ID2D1EffectContext>,
        transformgraph: Option<&ID2D1TransformGraph>,
    ) -> Result<()> {
        let effect_context = effectcontext.unwrap();
        let transform_graph = transformgraph.unwrap();
        unsafe {
            effect_context.LoadPixelShader(&SAMPLE_EFFECT_SHADER, shaders::threshold_shader())?;
            // Our base vtable is the IUnknown/IInspectable one
            let unknown = std::mem::transmute::<&Self, IUnknown>(self);
            let transform_node: ID2D1TransformNode = unknown.cast()?;
            transform_graph.SetSingleTransformNode(&transform_node)?;
        }
        Ok(())
    }

    fn PrepareForRender(&self, changetype: D2D1_CHANGE_TYPE) -> Result<()> {
        // TODO: Set pixel shader constant buffer
        todo!()
    }

    fn SetGraph(&self, transformgraph: Option<&ID2D1TransformGraph>) -> Result<()> {
        todo!()
    }
}

impl ID2D1DrawTransform_Impl for SampleEffect_Impl {
    fn SetDrawInfo(&self, drawinfo: Option<&ID2D1DrawInfo>) -> windows_core::Result<()> {
        todo!()
    }
}

impl ID2D1Transform_Impl for SampleEffect_Impl {
    fn MapOutputRectToInputRects(
        &self,
        outputrect: *const RECT,
        inputrects: *mut RECT,
        inputrectscount: u32,
    ) -> windows_core::Result<()> {
        todo!()
    }

    fn MapInputRectsToOutputRect(
        &self,
        inputrects: *const RECT,
        inputopaquesubrects: *const RECT,
        inputrectcount: u32,
        outputrect: *mut RECT,
        outputopaquesubrect: *mut RECT,
    ) -> windows_core::Result<()> {
        todo!()
    }

    fn MapInvalidRect(
        &self,
        inputindex: u32,
        invalidinputrect: &RECT,
    ) -> windows_core::Result<RECT> {
        todo!()
    }
}

impl ID2D1TransformNode_Impl for SampleEffect_Impl {
    fn GetInputCount(&self) -> u32 {
        todo!()
    }
}

impl SampleEffect {
    fn new() -> Self {
        Self { threshold: 0.0 }
    }
    pub fn register(factory: &ID2D1Factory1) -> Result<()> {
        let bindings = [D2D1_PROPERTY_BINDING {
            propertyName: w!("Threshold"),
            setFunction: Some(threshold_helpers::value_setter),
            getFunction: Some(threshold_helpers::value_getter),
        }];

        unsafe {
            factory.RegisterEffectFromString(
                &CLSID_SampleEffect,
                SAMPLE_EFFECT_XML,
                Some(&bindings),
                Some(Self::create_effect),
            )?;
        }
        Ok(())
    }
    unsafe extern "system" fn create_effect(effectimpl: *mut Option<IUnknown>) -> HRESULT {
        // This gets us the base vtable in SampleEffect_Impl
        let effect_unknown: IUnknown = Self::new().into();
        if let Some(effectimpl) = effectimpl.as_mut() {
            *effectimpl = Some(effect_unknown);
            S_OK
        } else {
            E_INVALIDARG
        }
    }
    fn set_threshold(&mut self, threshold: f32) -> Result<()> {
        self.threshold = threshold;
        Ok(())
    }
    fn get_threshold(&self) -> f32 {
        self.threshold
    }
}

// https://learn.microsoft.com/en-us/windows/win32/direct2d/custom-effects#define-a-public-registration-method
const SAMPLE_EFFECT_XML: PCWSTR = w!(r#"<?xml version='1.0'?>
<Effect>
    <!-- System Properties -->
    <Property name='DisplayName' type='string' value='SampleEffect'/>
    <Property name='Author' type='string' value='Contoso'/>
    <Property name='Category' type='string' value='Sample'/>
    <Property name='Description' type='string' value='This is a demo effect.'/>
    <Inputs>
        <Input name='SourceOne'/>
        <!-- <Input name='SourceTwo'/> -->
        <!-- Additional inputs go here. -->
    </Inputs>
    <!-- Custom Properties go here. -->
    <Property name='Threshold' type='float'>
        <Property name='DisplayName' type='string' value='Threshold'/>
        <Property name='Min' type='float' value='0.0' />
        <Property name='Max' type='float' value='1000.0' />
        <Property name='Default' type='float' value='0.0' />
    </Property>
</Effect>"#);

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

create_setter_helpers!(
    SampleEffect_Impl,
    set_threshold,
    get_threshold,
    f32,
    threshold_helpers
);

#[cfg(test)]
mod tests {
    use windows::{
        core::{IUnknown, Interface, Result},
        Win32::Graphics::Direct2D::ID2D1EffectImpl,
    };

    use super::SampleEffect;

    #[test]
    fn smoke() -> Result<()> {
        let unknown: IUnknown = SampleEffect::new().into();
        let effect_2: ID2D1EffectImpl = unknown.cast()?;
        let unknown_2: IUnknown = effect_2.cast()?;
        let effect_unknown: IUnknown = effect_2.into();
        Ok(())
    }
}
