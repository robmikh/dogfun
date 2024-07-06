use windows::{
    core::{implement, w, IUnknown, Result, GUID, HRESULT, PCWSTR},
    Win32::{
        Foundation::{E_INVALIDARG, RECT, S_OK},
        Graphics::Direct2D::{
            ID2D1DrawInfo, ID2D1DrawTransform, ID2D1DrawTransform_Impl, ID2D1EffectContext,
            ID2D1EffectImpl, ID2D1EffectImpl_Impl, ID2D1Factory1, ID2D1TransformGraph,
            ID2D1TransformNode, ID2D1TransformNode_Impl, ID2D1Transform_Impl, D2D1_CHANGE_TYPE,
            D2D1_PIXEL_OPTIONS_NONE, D2D1_PROPERTY_BINDING,
        },
    },
};
use windows_core::Interface;

pub const THRESHOLD_EFFECT_CLSID: GUID = GUID::from_u128(0xFB3AF5AA_6F03_4754_A676_BACB2D082069);
pub const THRESHOLD_EFFECT_SHADER: GUID = GUID::from_u128(0x397DBC73_8831_4C02_9ECD_56036DAEA108);

#[implement(ID2D1EffectImpl, ID2D1DrawTransform)]
pub struct ThresholdEffect {
    constants: ThresholdEffectConstants,
    draw_info: Option<ID2D1DrawInfo>,
}

#[repr(C)]
struct ThresholdEffectConstants {
    threshold: f32,
}

impl ID2D1EffectImpl_Impl for ThresholdEffect_Impl {
    fn Initialize(
        &self,
        effectcontext: Option<&ID2D1EffectContext>,
        transformgraph: Option<&ID2D1TransformGraph>,
    ) -> Result<()> {
        let effect_context = effectcontext.unwrap();
        let transform_graph = transformgraph.unwrap();
        unsafe {
            effect_context
                .LoadPixelShader(&THRESHOLD_EFFECT_SHADER, shaders::threshold_pixel_shader())?;
            // Our base vtable is the IUnknown/IInspectable one
            let unknown = std::mem::transmute::<&Self, IUnknown>(self);
            let transform_node: ID2D1TransformNode = unknown.cast()?;
            std::mem::forget(unknown); // Don't mess up our ref count
            transform_graph.SetSingleTransformNode(&transform_node)?;
        }
        Ok(())
    }

    fn PrepareForRender(&self, _changetype: D2D1_CHANGE_TYPE) -> Result<()> {
        if let Some(draw_info) = self.this.draw_info.as_ref() {
            unsafe {
                let len = std::mem::size_of::<ThresholdEffectConstants>();
                let slice =
                    std::slice::from_raw_parts(&self.this.constants as *const _ as *const u8, len);
                draw_info.SetPixelShaderConstantBuffer(slice)?;
            }
            Ok(())
        } else {
            panic!()
        }
    }

    fn SetGraph(&self, _transformgraph: Option<&ID2D1TransformGraph>) -> Result<()> {
        todo!()
    }
}

impl ID2D1DrawTransform_Impl for ThresholdEffect_Impl {
    fn SetDrawInfo(&self, drawinfo: Option<&ID2D1DrawInfo>) -> Result<()> {
        if let Some(draw_info) = drawinfo {
            unsafe {
                // TODO: Safely do this
                ((self as *const Self as *mut Self).as_mut().unwrap())
                    .this
                    .draw_info = Some(draw_info.clone());
                draw_info.SetPixelShader(&THRESHOLD_EFFECT_SHADER, D2D1_PIXEL_OPTIONS_NONE)?;
            }
            Ok(())
        } else {
            E_INVALIDARG.ok()
        }
    }
}

impl ID2D1Transform_Impl for ThresholdEffect_Impl {
    fn MapOutputRectToInputRects(
        &self,
        outputrect: *const RECT,
        inputrects: *mut RECT,
        inputrectscount: u32,
    ) -> Result<()> {
        if inputrectscount != 1 {
            return E_INVALIDARG.ok();
        }

        let output_rect = unsafe {
            outputrect
                .as_ref()
                .map(|x| Ok(x))
                .unwrap_or(Err(E_INVALIDARG))?
        };
        let input_rect = unsafe {
            inputrects
                .as_mut()
                .map(|x| Ok(x))
                .unwrap_or(Err(E_INVALIDARG))?
        };
        *input_rect = *output_rect;

        Ok(())
    }

    fn MapInputRectsToOutputRect(
        &self,
        inputrects: *const RECT,
        _inputopaquesubrects: *const RECT,
        inputrectcount: u32,
        outputrect: *mut RECT,
        outputopaquesubrect: *mut RECT,
    ) -> Result<()> {
        if inputrectcount != 1 {
            return E_INVALIDARG.ok();
        }

        let input_rect = unsafe {
            inputrects
                .as_ref()
                .map(|x| Ok(x))
                .unwrap_or(Err(E_INVALIDARG))?
        };
        let output_rect = unsafe {
            outputrect
                .as_mut()
                .map(|x| Ok(x))
                .unwrap_or(Err(E_INVALIDARG))?
        };
        *output_rect = *input_rect;
        let output_opaque_rect = unsafe {
            outputopaquesubrect
                .as_mut()
                .map(|x| Ok(x))
                .unwrap_or(Err(E_INVALIDARG))?
        };
        *output_opaque_rect = *input_rect;

        Ok(())
    }

    fn MapInvalidRect(&self, _inputindex: u32, _invalidinputrect: &RECT) -> Result<RECT> {
        todo!()
    }
}

impl ID2D1TransformNode_Impl for ThresholdEffect_Impl {
    fn GetInputCount(&self) -> u32 {
        1
    }
}

impl ThresholdEffect {
    fn new() -> Self {
        Self {
            constants: ThresholdEffectConstants { threshold: 0.0 },
            draw_info: None,
        }
    }
    pub fn register(factory: &ID2D1Factory1) -> Result<()> {
        let bindings = [D2D1_PROPERTY_BINDING {
            propertyName: w!("Threshold"),
            setFunction: Some(threshold_helpers::value_setter),
            getFunction: Some(threshold_helpers::value_getter),
        }];

        unsafe {
            factory.RegisterEffectFromString(
                &THRESHOLD_EFFECT_CLSID,
                SAMPLE_EFFECT_XML,
                Some(&bindings),
                Some(Self::create_effect),
            )?;
        }
        Ok(())
    }
    unsafe extern "system" fn create_effect(effectimpl: *mut Option<IUnknown>) -> HRESULT {
        // This gets us the base vtable in ThresholdEffect_Impl
        let effect_unknown: IUnknown = Self::new().into();
        if let Some(effectimpl) = effectimpl.as_mut() {
            *effectimpl = Some(effect_unknown);
            S_OK
        } else {
            E_INVALIDARG
        }
    }
    fn set_threshold(&mut self, threshold: f32) -> Result<()> {
        self.constants.threshold = threshold;
        Ok(())
    }
    fn get_threshold(&self) -> f32 {
        self.constants.threshold
    }
}

// https://learn.microsoft.com/en-us/windows/win32/direct2d/custom-effects#define-a-public-registration-method
const SAMPLE_EFFECT_XML: PCWSTR = w!(r#"<?xml version='1.0'?>
<Effect>
    <!-- System Properties -->
    <Property name='DisplayName' type='string' value='ThresholdEffect'/>
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

create_setter_helpers!(
    ThresholdEffect_Impl,
    set_threshold,
    get_threshold,
    f32,
    threshold_helpers
);
