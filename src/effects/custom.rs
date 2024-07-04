use windows::{core::{implement, interface, IUnknown, Result, GUID, HRESULT, HSTRING, PCWSTR, w}, Win32::{Foundation::{E_INVALIDARG, S_OK}, Graphics::Direct2D::{ID2D1EffectContext, ID2D1EffectImpl, ID2D1EffectImpl_Impl, ID2D1Factory1, ID2D1TransformGraph, D2D1_CHANGE_TYPE, PD2D1_EFFECT_FACTORY}}};

pub const CLSID_SampleEffect: GUID = GUID::from_u128(0xFB3AF5AA_6F03_4754_A676_BACB2D082069);

#[implement(ID2D1EffectImpl)]
pub struct SampleEffect {

}


impl ID2D1EffectImpl_Impl for SampleEffect_Impl {
    fn Initialize(&self,effectcontext:Option<&ID2D1EffectContext>,transformgraph:Option<&ID2D1TransformGraph>) -> windows_core::Result<()> {
        todo!()
    }

    fn PrepareForRender(&self,changetype:D2D1_CHANGE_TYPE) -> windows_core::Result<()> {
        todo!()
    }

    fn SetGraph(&self,transformgraph:Option<&ID2D1TransformGraph>) -> windows_core::Result<()> {
        todo!()
    }
}

impl SampleEffect {
    fn new() -> Self { Self {} }
    fn register(factory: &ID2D1Factory1) -> Result<()> {
        unsafe {
            factory.RegisterEffectFromString(&CLSID_SampleEffect, SAMPLE_EFFECT_XML, None, Some(Self::create_effect))?;
        }
        Ok(())
    }
    unsafe extern "system" fn create_effect(effectimpl: *mut Option<IUnknown>) -> HRESULT {
        let effect: ID2D1EffectImpl = Self::new().into();
        if let Some(effectimpl) = effectimpl.as_mut() {
            let effect_unknown: IUnknown = effect.into();
            *effectimpl = Some(effect_unknown);
            S_OK
        } else {
            E_INVALIDARG
        }
    }
}

// https://learn.microsoft.com/en-us/windows/win32/direct2d/custom-effects#define-a-public-registration-method
const SAMPLE_EFFECT_XML: PCWSTR = w!(
r#"#define XML(X) TEXT(#X) // This macro creates a single string from multiple lines of text.

PCWSTR pszXml =
    XML(
        <?xml version='1.0'?>
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
        </Effect>
        );"#);