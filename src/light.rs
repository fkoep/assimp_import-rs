use prim::{self, Color3, Vector2, Vector3};
use ffi;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum LightSourceType {
    Undefined = 0x0,

    /// A directional light source has a well-defined direction
    /// but is infinitely far away. That's quite a good
    /// approximation for sun light.
    Directional = 0x1,

    /// A point light source has a well-defined position
    /// in space but no direction - it emits light in all
    /// directions. A normal bulb is a point light.
    Point = 0x2,

    /// A spot light source emits light in a specific
    /// angle. It has a position and a direction it is pointing to.
    /// A good example for a spot light is a light spot in
    /// sport arenas.
    Spot = 0x3,

    /// The generic light level of the world, including the bounces
    /// of all other light sources.
    /// Typically, there's at most one ambient light in a scene.
    /// This light type doesn't have a valid position, direction, or
    /// other properties, just a color.
    Ambient = 0x4,

    /// An area light is a rectangle with predefined size that uniformly
    /// emits light from one of its sides. The position is center of the
    /// rectangle and direction is its normal vector.
    Area = 0x5,
}
ai_impl_enum!(LightSourceType, ffi::aiLightSourceType);

ai_ptr_type!{
    /// Helper structure to describe a light source.
    ///
    /// Assimp supports multiple sorts of light sources, including
    /// directional, point and spot lights. All of them are defined with just
    /// a single structure and distinguished by their parameters.
    /// Note - some file formats (such as 3DS, ASE) export a "target point" -
    /// the point a spot light is looking at (it can even be animated). Assimp
    /// writes the target point as a subnode of a spotlights's main node,
    /// called "<spotName>.Target". However, this is just additional information
    /// then, the transformation tracks of the main node make the
    /// spot light already point in the right direction.
    type Light: ffi::aiLight;
}

impl<'a> Light<'a> {
    /// The name of the light source.
    ///
    /// There must be a node in the scenegraph with the same name.
    /// This node specifies the position of the light in the scene
    /// hierarchy and can be animated.
    pub fn name(&self) -> &str {
        prim::str(&self.raw().mName).unwrap()
    }

    /// The type of the light source.
    pub fn source_type(&self) -> LightSourceType {
        unsafe { LightSourceType::from_ffi(self.raw().mType) }
    }

    /// Position of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The position is undefined for directional lights.
    pub fn position(&self) -> Vector3 {
        prim::vec3(self.raw().mPosition)
    }

    /// Direction of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The direction is undefined for point lights. The vector
    /// may be normalized, but it needn't.
    pub fn direction(&self) -> Vector3 {
        prim::vec3(self.raw().mDirection)
    }

    /// Up direction of the light source in space. Relative to the
    /// transformation of the node corresponding to the light.
    ///
    /// The direction is undefined for point lights. The vector
    /// may be normalized, but it needn't.
    pub fn up(&self) -> Vector3 {
        prim::vec3(self.raw().mUp)
    }

    /// Constant light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// @code
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// @endcode
    /// This member corresponds to the att0 variable in the equation.
    /// Naturally undefined for directional lights.
    pub fn attenuation_constant(&self) -> f32 {
        self.raw().mAttenuationConstant
    }

    /// Linear light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// @code
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// @endcode
    /// This member corresponds to the att1 variable in the equation.
    /// Naturally undefined for directional lights.
    pub fn attenuation_linear(&self) -> f32 {
        self.raw().mAttenuationLinear
    }

    /// Quadratic light attenuation factor.
    ///
    /// The intensity of the light source at a given distance 'd' from
    /// the light's position is
    /// @code
    /// Atten = 1/( att0 + att1 * d + att2 * d*d)
    /// @endcode
    /// This member corresponds to the att2 variable in the equation.
    /// Naturally undefined for directional lights.
    pub fn attenuation_quadratic(&self) -> f32 {
        self.raw().mAttenuationQuadratic
    }

    /// Diffuse color of the light source
    ///
    /// The diffuse light color is multiplied with the diffuse
    /// material color to obtain the final color that contributes
    /// to the diffuse shading term.
    pub fn color_diffuse(&self) -> Color3 {
        prim::col3(self.raw().mColorDiffuse)
    }

    /// Specular color of the light source
    ///
    /// The specular light color is multiplied with the specular
    /// material color to obtain the final color that contributes
    /// to the specular shading term.
    pub fn color_specular(&self) -> Color3 {
        prim::col3(self.raw().mColorSpecular)
    }

    /// Ambient color of the light source
    ///
    /// The ambient light color is multiplied with the ambient
    /// material color to obtain the final color that contributes
    /// to the ambient shading term. Most renderers will ignore
    /// this value it, is just a remaining of the fixed-function pipeline
    /// that is still supported by quite many file formats.
    pub fn color_ambient(&self) -> Color3 {
        prim::col3(self.raw().mColorAmbient)
    }

    /// Inner angle of a spot light's light cone.
    ///
    /// The spot light has maximum influence on objects inside this
    /// angle. The angle is given in radians. It is 2PI for point
    /// lights and undefined for directional lights.
    pub fn angle_inner_cone(&self) -> f32 {
        self.raw().mAngleInnerCone
    }

    /// Outer angle of a spot light's light cone.
    ///
    /// The spot light does not affect objects outside this angle.
    /// The angle is given in radians. It is 2PI for point lights and
    /// undefined for directional lights. The outer angle must be
    /// greater than or equal to the inner angle.
    /// It is assumed that the application uses a smooth
    /// interpolation between the inner and the outer cone of the
    /// spot light.
    pub fn angle_outer_cone(&self) -> f32 {
        self.raw().mAngleOuterCone
    }

    /// Size of area light source.
    pub fn size(&self) -> Vector2 {
        prim::vec2(self.raw().mSize)
    }
}
