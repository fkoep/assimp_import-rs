use prim::{self, Texel};
use ffi;
use std::ffi::CStr;
use std::str;

ai_ptr_type!{
    /// Helper structure to describe an embedded texture
    ///
    /// Normally textures are contained in external files but some file formats embed
    /// them directly in the model file. There are two prim of embedded textures:
    ///
    /// 1. Uncompressed textures. The color data is given in an uncompressed format.
    /// 2. Compressed textures stored in a file format like png or jpg. The raw file
    /// bytes are given so the application must utilize an image decoder (e.g. DevIL) to
    /// get access to the actual color data.
    ///
    /// Embedded textures are referenced from materials using strings like "*0", "*1", etc.
    /// as the texture paths (a single asterisk character followed by the
    /// zero-based index of the texture in the aiScene::mTextures array).
    type Texture: ffi::aiTexture;
}

impl<'a> Texture<'a> {
    /// A hint from the loader to make it easier for applications
    ///  to determine the type of embedded compressed textures.
    ///
    /// If mHeight != 0 this member is undefined. Otherwise it
    /// is set set to '\\0\\0\\0\\0' if the loader has no additional
    /// information about the texture file format used OR the
    /// file extension of the format without a trailing dot. If there
    /// are multiple file extensions for a format, the shortest
    /// extension is chosen (JPEG maps to 'jpg', not to 'jpeg').
    /// E.g. 'dds\\0', 'pcx\\0', 'jpg\\0'.  All characters are lower-case.
    /// The fourth character will always be '\\0'.
    pub fn format_hint(&self) -> Option<&str> {
        if self.raw().mHeight != 0 {
            return None;
        }
        unsafe { CStr::from_ptr(self.raw().achFormatHint.as_ptr()).to_str().ok() }
    }

    pub fn as_texels(&self) -> Option<(usize, usize, &[Texel])> {
        let (w, h) = (self.raw().mWidth, self.raw().mHeight);
        if h == 0 {
            return None;
        }
        let len = w * h;
        let texels = unsafe { prim::slice(self.raw().pcData as *const ffi::aiTexel, len) };
        Some((w as usize, h as usize, texels))
    }
    pub fn as_bytes(&self) -> &[u8] {
        let (w, h) = (self.raw().mWidth, self.raw().mHeight);
        let len = if h == 0 { w } else { h * w * 4 };
        unsafe { prim::slice(self.raw().pcData as *const u8, len) }
    }
}
