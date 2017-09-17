use prim::{self, Color4, Vector2, Vector3};
use ffi;
use std::{mem, ptr, slice, str};
use libc::{c_uint, c_int, c_char};

pub type TextureIdx = c_uint;

/// Name for default materials (2nd is used if meshes have UV coords)
pub const DEFAULT_MATERIAL_NAME: &'static str = "DefaultMaterial";

/// Defines how the Nth texture of a specific type is combined with
/// the result of all previous layers.
///
/// Example (left: key, right: value):
///
/// ```
/// DiffColor0     - gray
/// DiffTextureOp0 - aiTextureOpMultiply
/// DiffTexture0   - tex1.png
/// DiffTextureOp0 - aiTextureOpAdd
/// DiffTexture1   - tex2.png
/// ```
///
/// Written as equation, the final diffuse term for a specific pixel would be:
///
/// ```
/// diffFinal = DiffColor0 * sampleTex(DiffTexture0,UV0) +
///    sampleTex(DiffTexture1,UV0) * diffContrib;
/// ```
///
/// where 'diffContrib' is the intensity of the incoming light for that pixel.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureOp {
    /// `T = T1 * T2`
    Multiply = 0x0,

    /// `T = T1 + T2`
    Add = 0x1,

    /// `T = T1 - T2`
    Subtract = 0x2,

    /// `T = T1 / T2`
    Divide = 0x3,

    /// `T = (T1 + T2) - (T1 * T2)`
    SmoothAdd = 0x4,

    /// `T = T1 + (T2 - 0.5)`
    SignedAdd = 0x5,
}
ai_impl_enum!(TextureOp, c_uint);

/// Defines how UV coordinates outside the [0...1] range are handled.
///
/// Commonly referred to as 'wrapping mode'.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureMapMode {
    /// A texture coordinate u|v is translated to u%1|v%1
    Wrap = 0x0,

    /// Texture coordinates outside [0...1]
    /// are clamped to the nearest valid value.
    Clamp = 0x1,

    /// If the texture coordinates for a pixel are outside [0...1]
    /// the texture is not applied to that pixel
    Decal = 0x3,

    /// A texture coordinate u|v becomes u%1|v%1 if (u-(u%1))%2 is zero and
    /// 1-(u%1)|1-(v%1) otherwise
    Mirror = 0x2,
}
ai_impl_enum!(TextureMapMode, c_uint);

/// Defines how the mapping coords for a texture are generated.
///
/// Real-time applications typically require full UV coordinates, so the use of
/// the aiProcess_GenUVCoords step is highly recommended. It generates proper
/// UV channels for non-UV mapped objects, as long as an accurate description
/// how the mapping should look like (e.g spherical) is given.
/// See the #AI_MATKEY_MAPPING property for more details.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureMapping {
    /// The mapping coordinates are taken from an UV channel.
    ///
    /// The #AI_MATKEY_UVWSRC key specifies from which UV channel
    /// the texture coordinates are to be taken from (remember,
    /// meshes can have more than one UV channel).
    Uv = 0x0,

    /// Spherical mapping
    Sphere = 0x1,

    /// Cylindrical mapping
    Cylinder = 0x2,

    /// Cubic mapping
    Box = 0x3,

    /// Planar mapping
    Plane = 0x4,

    /// Undefined mapping. Have fun.
    Other = 0x5,
}
ai_impl_enum!(TextureMapping, c_uint);

/// Defines the purpose of a texture
///
/// This is a very difficult topic. Different 3D packages support different
/// kinds of textures. For very common texture prim, such as bumpmaps, the
/// rendering results depend on implementation details in the rendering
/// pipelines of these applications. Assimp loads all texture references from
/// the model file and tries to determine which of the predefined texture
/// prim below is the best choice to match the original use of the texture
/// as closely as possible.
///
/// In content pipelines you'll usually define how textures have to be handled,
/// and the artists working on models have to conform to this specification,
/// regardless which 3D tool they're using.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    /// Dummy value.
    ///
    /// No texture, but the value to be used as 'texture semantic'
    /// (#aiMaterialProperty::mSemantic) for all material properties
    /// *not* related to textures.
    None = 0x0,

    /// The texture is combined with the result of the diffuse
    /// lighting equation.
    Diffuse = 0x1,

    /// The texture is combined with the result of the specular
    /// lighting equation.
    Specular = 0x2,

    /// The texture is combined with the result of the ambient
    /// lighting equation.
    Ambient = 0x3,

    /// The texture is added to the result of the lighting
    /// calculation. It isn't influenced by incoming light.
    Emissive = 0x4,

    /// The texture is a height map.
    ///
    /// By convention, higher gray-scale values stand for
    /// higher elevations from the base height.
    Height = 0x5,

    /// The texture is a (tangent space) normal-map.
    ///
    /// Again, there are several conventions for tangent-space
    /// normal maps. Assimp does (intentionally) not
    /// distinguish here.
    Normals = 0x6,

    /// The texture defines the glossiness of the material.
    ///
    /// The glossiness is in fact the exponent of the specular
    /// (phong) lighting equation. Usually there is a conversion
    /// function defined to map the linear color values in the
    /// texture to a suitable exponent. Have fun.
    Shininess = 0x7,

    /// The texture defines per-pixel opacity.
    ///
    /// Usually 'white' means opaque and 'black' means
    /// 'transparency'. Or quite the opposite. Have fun.
    Opacity = 0x8,

    /// Displacement texture
    ///
    /// The exact purpose and format is application-dependent.
    /// Higher color values stand for higher vertex displacements.
    Displacement = 0x9,

    /// Lightmap texture (aka Ambient Occlusion)
    ///
    /// Both 'Lightmaps' and dedicated 'ambient occlusion maps' are
    /// covered by this material property. The texture contains a
    /// scaling value for the final color value of a pixel. Its
    /// intensity is not affected by incoming light.
    Lightmap = 0xA,

    /// Reflection texture
    ///
    /// Contains the color of a perfect mirror reflection.
    /// Rarely used, almost never for real-time applications.
    Reflection = 0xB,

    /// Unknown texture
    ///
    /// A texture reference that does not match any of the definitions
    /// above is considered to be 'unknown'. It is still imported,
    /// but is excluded from any further postprocessing.
    Unknown = 0xC,
}
ai_impl_enum!(TextureType, c_uint);

/// Defines all shading models supported by the library
///
/// The list of shading modes has been taken from Blender.
/// See Blender documentation for more information. The API does
/// not distinguish between "specular" and "diffuse" shaders (thus the
/// specular term for diffuse shading models like Oren-Nayar remains
/// undefined). <br>
/// Again, this value is just a hint. Assimp tries to select the shader whose
/// most common implementation matches the original rendering results of the
/// 3D modeller which wrote a particular model as closely as possible.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ShadingMode {
    /// Flat shading. Shading is done on per-face base,
    /// diffuse only. Also known as 'faceted shading'.
    Flat = 0x1,

    /// Simple Gouraud shading.
    Gouraud = 0x2,

    /// Phong-Shading
    Phong = 0x3,

    /// Phong-Blinn-Shading
    Blinn = 0x4,

    /// Toon-Shading per pixel
    ///
    /// Also known as 'comic' shader.
    Toon = 0x5,

    /// OrenNayar-Shading per pixel
    ///
    /// Extension to standard Lambertian shading, taking the
    /// roughness of the material into account
    OrenNayar = 0x6,

    /// Minnaert-Shading per pixel
    ///
    /// Extension to standard Lambertian shading, taking the
    /// "darkness" of the material into account
    Minnaert = 0x7,

    /// CookTorrance-Shading per pixel
    ///
    /// Special shader for metallic surfaces.
    CookTorrance = 0x8,

    /// No shading at all. Constant light influence of 1.0.
    NoShading = 0x9,

    /// Fresnel shading
    Fresnel = 0xA,
}
ai_impl_enum!(ShadingMode, c_uint);

bitflags!{
    /// Defines some mixed flags for a particular texture.
    ///
    /// Usually you'll instruct your cg artists how textures have to look like ...
    /// and how they will be processed in your application. However, if you use
    /// Assimp for completely generic loading purposes you might also need to
    /// process these flags in order to display as many 'unknown' 3D models as
    /// possible correctly.
    ///
    /// This corresponds to the #AI_MATKEY_TEXFLAGS property.
    pub flags TextureFlags: c_uint {

        /// The texture's color values have to be inverted (componentwise 1-n)
        const INVERT = 0x1,

        /// Explicit request to the application to process the alpha channel
        /// of the texture.
        ///
        /// Mutually exclusive with #aiTextureFlags_IgnoreAlpha. These
        /// flags are set if the library can say for sure that the alpha
        /// channel is used/is not used. If the model format does not
        /// define this, it is left to the application to decide whether
        /// the texture alpha channel - if any - is evaluated or not.
        const USE_ALPHA = 0x2,

        /// Explicit request to the application to ignore the alpha channel
        /// of the texture.
        ///
        /// Mutually exclusive with #aiTextureFlags_UseAlpha.
        const IGNORE_ALPHA = 0x4,
    }
}

/// Defines alpha-blend flags.
///
/// If you're familiar with OpenGL or D3D, these flags aren't new to you.
/// They define *how* the final color value of a pixel is computed, basing
/// on the previous color at that pixel and the new color value from the
/// material.
/// The blend formula is:
///
/// ```
///   SourceColor * SourceBlend + DestColor * DestBlend
/// ```
///
/// where DestColor is the previous color in the framebuffer at this
/// position and SourceColor is the material colro before the transparency
/// calculation.
///
/// This corresponds to the #AI_MATKEY_BLEND_FUNC property.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    /// `SourceColor * SourceAlpha + DestColor * (1 - SourceAlpha)`
    Default = 0x0,

    /// `SourceColor + DestColor`
    Additive = 0x1,
}
ai_impl_enum!(BlendMode, c_uint);

ai_type! {
    /// Defines how an UV channel is transformed.
    ///
    /// This is just a helper structure for the #AI_MATKEY_UVTRANSFORM key.
    /// See its documentation for more details.
    ///
    /// Typically you'll want to build a matrix of this information. However,
    /// we keep separate scaling/translation/rotation values to make it
    /// easier to process and optimize UV transformations internally.
    #[derive(Debug, Clone, Copy)]
    type UvTransform: ffi::aiUVTransform;
}

impl UvTransform {
    /// Translation on the u and v axes.
    ///
    /// The default value is (0|0).
    pub fn translation(&self) -> Vector2 {
        prim::vec2(self.raw.mTranslation)
    }

    /// Scaling on the u and v axes.
    ///
    /// The default value is (1|1).
    pub fn scaling(&self) -> Vector2 {
        prim::vec2(self.raw.mScaling)
    }

    /// Rotation - in counter-clockwise direction.
    ///
    /// The rotation angle is specified in radians. The
    /// rotation center is 0.5f|0.5f. The default value
    /// 0.f.
    pub fn rotation(&self) -> f32 {
        self.raw.mRotation
    }
}

/*
#[derive(Debug, Clone, Copy)]
pub enum MaterialPropertyData<'a> {
    Float(&'a [f32]),
    String(&'a str),
    Integer(&'a [u32]),
    Buffer(&'a [u8]),
}

ai_ptr_type!{

    type MaterialProperty: ffi::aiMaterialProperty;
}

impl<'a> MaterialProperty<'a> {
    pub fn key(&self) -> &str {
        prim::str(&self.raw().mKey)
    }
    pub fn idx(&self) -> TextureIdx {
        self.raw().mIndex
    }

    pub fn semantic(&self) -> TextureType {
        unsafe { TextureType::from_ffi(self.raw().mSemantic) }
    }

    pub fn data(&self) -> MaterialPropertyData {
        use ffi::aiPropertyTypeInfo::*;

        match self.raw().mType {
            aiPTI_Float => {
                let ret = unsafe { prim::transmute_slice(self.raw().mData, self.raw().mDataLength / 4) };
                MaterialPropertyData::Float(ret)
            }
            aiPTI_String => {
                let ret = unsafe {
                    let bytes = prim::transmute_slice(self.raw().mData, self.raw().mDataLength - 1); // TODO -1 for zero byte needed?
                    str::from_utf8(&bytes).unwrap()
                };
                MaterialPropertyData::String(ret)
            }
            aiPTI_Integer => {
                let ret = unsafe { prim::transmute_slice(self.raw().mData, self.raw().mDataLength / 4) };
                MaterialPropertyData::Integer(ret)
            }
            aiPTI_Buffer => {
                let ret = unsafe { prim::transmute_slice(self.raw().mData, self.raw().mDataLength) };
                MaterialPropertyData::Buffer(ret)
            }
            _ => unreachable!(),
        }
    }
}
*/

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub name: String,
    pub twosided: bool,
    pub shading_mode: ShadingMode,
    pub wireframe: bool,
    pub blend_mode: BlendMode,
    pub opacity: f32,
    pub bumpscaling: f32,
    pub shininess: f32,
    pub shininess_strength: f32,
    pub reflectivity: f32,
    pub refracti: f32,
    /* TODO should these be Color3? */
    pub color_diffuse: Color4,
    pub color_ambient: Color4,
    pub color_specular: Color4,
    pub color_emissive: Color4,
    pub color_transparent: Color4,
    pub color_reflective: Color4,
    //TODO pub other: BTreeMap<String, ?>,
}

// TODO
//pub enum TextureRef {
//  Embedded(TextureIdx),
//  External(PathBuf),
//}

/// TODO
#[derive(Debug, Clone)]
pub struct TextureProperties {
    pub texture_ref: String,
    pub mapping: TextureMapping,
    pub uv_index: Option<u32>, 
    pub blend: f32,
    pub op: TextureOp,
    pub map_mode: [TextureMapMode; 2],
    pub flags: TextureFlags,
    //TODO pub other: BTreeMap<String, ?>,
}

ai_ptr_type!{
    /// TODO Docs
    type Material: ffi::aiMaterial;
}

impl<'a> Material<'a> {
    /* TODO?
	pub fn properties(&self) -> &[MaterialProperty] {
		unsafe { prim::slice(self.raw().mProperties, self.raw().mNumProperties) }
	}
    */

    pub fn material_properties(&self) -> MaterialProperties {
        let mut name = ffi::aiString::default();
        let mut twosided: c_int = 0;
        let mut shading_mode: c_int = ShadingMode::Gouraud as u32 as i32;
        let mut wireframe: c_int = 0;
        let mut blend_mode: c_int = 0;
        let mut opacity = 1.0;
        let mut bumpscaling = 0.0;
        let mut shininess = 0.0;
        let mut shininess_strength = 1.0;
        let mut reflectivity = 0.0; // TODO default?
        let mut refracti = 1.0;
        let mut color_diffuse = ffi::aiColor4D::default();
        let mut color_ambient = ffi::aiColor4D::default();
        let mut color_specular = ffi::aiColor4D::default();
        let mut color_emissive = ffi::aiColor4D::default();
        let mut color_transparent = ffi::aiColor4D::default();
        let mut color_reflective = ffi::aiColor4D::default(); // TODO default?

        unsafe {
            ffi::aiGetMaterialString(
                self.as_ptr(), "?mat.name\0".as_ptr() as *const c_char, 0, 0, &mut name
            );
            ffi::aiGetMaterialIntegerArray(
                self.as_ptr(), "$mat.twosided\0".as_ptr() as *const c_char, 0, 0, &mut twosided, ptr::null_mut()
            );
            ffi::aiGetMaterialIntegerArray(
                self.as_ptr(), "$mat.shadingm\0".as_ptr() as *const c_char, 0, 0, &mut shading_mode, ptr::null_mut()
            );
            ffi::aiGetMaterialIntegerArray(
                self.as_ptr(), "$mat.wireframe\0".as_ptr() as *const c_char, 0, 0, &mut wireframe, ptr::null_mut()
            );
            ffi::aiGetMaterialIntegerArray(
                self.as_ptr(), "$mat.blend\0".as_ptr() as *const c_char, 0, 0, &mut blend_mode, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.opacity\0".as_ptr() as *const c_char, 0, 0, &mut opacity, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.bumpscaling\0".as_ptr() as *const c_char, 0, 0, &mut bumpscaling, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.shininess\0".as_ptr() as *const c_char, 0, 0, &mut shininess, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.shinpercent\0".as_ptr() as *const c_char, 0, 0, &mut shininess_strength, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.reflectivity\0".as_ptr() as *const c_char, 0, 0, &mut reflectivity, ptr::null_mut()
            );
            ffi::aiGetMaterialFloatArray(
                self.as_ptr(), "$mat.refracti\0".as_ptr() as *const c_char, 0, 0, &mut refracti, ptr::null_mut()
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.diffuse\0".as_ptr() as *const c_char, 0, 0, &mut color_diffuse
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.ambient\0".as_ptr() as *const c_char, 0, 0, &mut color_ambient
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.specular\0".as_ptr() as *const c_char, 0, 0, &mut color_specular
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.emissive\0".as_ptr() as *const c_char, 0, 0, &mut color_emissive
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.transparent\0".as_ptr() as *const c_char, 0, 0, &mut color_transparent
            );
            ffi::aiGetMaterialColor(
                self.as_ptr(), "$clr.reflective\0".as_ptr() as *const c_char, 0, 0, &mut color_reflective
            );

            MaterialProperties {
                name: prim::str(&name).unwrap().to_owned(),
                twosided: twosided != 0,
                shading_mode: ShadingMode::from_ffi(shading_mode as c_uint),
                wireframe: wireframe != 0,
                blend_mode: BlendMode::from_ffi(blend_mode as c_uint),
                opacity,
                bumpscaling,
                shininess,
                shininess_strength,
                reflectivity,
                refracti,
                color_diffuse: prim::col4(color_diffuse),
                color_ambient: prim::col4(color_ambient),
                color_specular: prim::col4(color_specular),
                color_emissive: prim::col4(color_emissive),
                color_transparent: prim::col4(color_transparent),
                color_reflective: prim::col4(color_reflective),
            }
        }
    }

    pub fn count_texture_properties(&self, tex_ty: TextureType) -> u32 {
        unsafe {
            ffi::aiGetMaterialTextureCount(
                self.as_ptr(), 
                mem::transmute::<_, ffi::aiTextureType>(tex_ty as u32) // FIXME remove transmute
            ) as u32
        }
    }

    pub fn texture_properties(&self, tex_ty: TextureType, idx: u32) -> Option<TextureProperties> {
        if idx >= self.count_texture_properties(tex_ty) {
            return None
        }

        let mut path = ffi::aiString::default();
        let mut mapping = ffi::aiTextureMapping::aiTextureMapping_OTHER; // TODO Default?
        let mut uv_index: c_uint = !0;
        let mut blend = 0.0; // TODO Default?
        let mut op = ffi::aiTextureOp::aiTextureOp_Multiply; // TODO Default?
        let mut map_mode = [ffi::aiTextureMapMode::aiTextureMapMode_Wrap; 2]; // TODO Default?
        let mut flags: c_uint = 0; // TODO Default?

        unsafe {
            use ffi::aiReturn::aiReturn_SUCCESS;

            let ok = ffi::aiGetMaterialTexture(
                self.as_ptr(), 
                mem::transmute::<_, ffi::aiTextureType>(tex_ty as u32), // FIXME remove transmute
                idx,
                &mut path,
                &mut mapping,
                &mut uv_index,
                &mut blend,
                &mut op,
                map_mode.as_mut_ptr(),
                &mut flags,
            ) == aiReturn_SUCCESS;

            if ok {
                Some(TextureProperties {
                    texture_ref: prim::str(&path).unwrap().to_owned(),
                    mapping: TextureMapping::from_ffi(mapping as u32), 
                    uv_index: if uv_index != !0 { Some(uv_index) } else { None },
                    blend,
                    op: TextureOp::from_ffi(op as u32), 
                    map_mode: [TextureMapMode::from_ffi(map_mode[0] as u32), TextureMapMode::from_ffi(map_mode[1] as u32)],
                    flags: TextureFlags::from_bits(flags).unwrap(),
                })
            } else {
                None
            }
        }
    }
}

