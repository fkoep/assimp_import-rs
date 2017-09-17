use anim::Animation;
use camera::Camera;
use light::Light;
use material::Material;
use metadata::MetaData;
use mesh::Mesh;
use postprocess::PostProcessSteps;
use texture::Texture;
use prim::{self, Matrix4};
use ffi;
use std::ffi::CStr;
use libc::c_uint;

// ++++++++++++++++++++ Node ++++++++++++++++++++

pub type MeshIdx = c_uint;

ai_ptr_type!{
    /// A node in the imported hierarchy.
    ///
    /// Each node has name, a parent node (except for the root node),
    /// a transformation relative to its parent and possibly several child nodes.
    /// Simple file formats don't support hierarchical structures - for these formats
    /// the imported scene does consist of only a single root node without children.
    type Node: ffi::aiNode;
}

impl<'a> Node<'a> {
    /// The name of the node.
    ///
    /// The name might be empty (length of zero) but all nodes which
    /// need to be referenced by either bones or animations are named.
    /// Multiple nodes may have the same name, except for nodes which are referenced
    /// by bones (see #aiBone and #aiMesh::mBones). Their names *must* be unique.
    ///
    /// Cameras and lights reference a specific node by name - if there
    /// are multiple nodes with this name, they are assigned to each of them.
    /// <br>
    /// There are no limitations with regard to the characters contained in
    /// the name string as it is usually taken directly from the source file.
    ///
    /// Implementations should be able to handle tokens such as whitespace, tabs,
    /// line feeds, quotation marks, ampersands etc.
    ///
    /// Sometimes assimp introduces new nodes not present in the source file
    /// into the hierarchy (usually out of necessity because sometimes the
    /// source hierarchy format is simply not compatible). Their names are
    /// surrounded by @verbatim <> @endverbatim e.g.
    ///  @verbatim<DummyRootNode> @endverbatim.
    pub fn name(&self) -> Option<&str> {
        prim::str(&self.raw().mName)
    }

    /// The transformation relative to the node's parent.
    pub fn transform(&self) -> Matrix4 {
        prim::mat4(self.raw().mTransformation)
    }

    /// Parent node. NULL if this node is the root node.
    pub fn parent(&self) -> Option<Self> {
        if self.raw().mParent.is_null() {
            return None;
        }
        unsafe { Some(Self::from_ptr(self.raw().mParent)) }
    }

    /// The child nodes of this node. NULL if mNumChildren is 0.
    pub fn children(&self) -> &[Self] {
        unsafe { Self::slice(self.raw().mChildren, self.raw().mNumChildren) }
    }

    /// The meshes of this node.
    ///
    /// Each entry is an index into the  mesh list of the #aiScene.
    pub fn meshes(&self) -> &[MeshIdx] {
        unsafe { prim::slice(self.raw().mMeshes, self.raw().mNumMeshes) }
    }

    /// Metadata associated with this node or NULL if there is no metadata.
    ///
    /// Whether any metadata is generated depends on the source file format. See the
    /// @link importer_notes @endlink page for more information on every source file
    /// format. Importers that don't document any metadata don't write any.
    pub fn meta_data(&self) -> Option<MetaData<'a>> {
        if self.raw().mMetaData.is_null() {
            return None;
        }
        unsafe { Some(MetaData::from_ptr(self.raw().mMetaData)) }
    }
}

// ++++++++++++++++++++ Scene ++++++++++++++++++++

bitflags!{
    pub flags SceneFlags: c_uint {
        /// Specifies that the scene data structure that was imported is not complete.
        ///
        /// This flag bypasses some internal validations and allows the import
        /// of animation skeletons, material libraries or camera animation paths
        /// using Assimp. Most applications won't support such data.
        const INCOMPLETE = 0x1,

        /// This flag is set by the validation postprocess-step (aiPostProcess_ValidateDS)
        /// if the validation is successful.
        ///
        /// In a validated scene you can be sure that
        /// any cross references in the data structure (e.g. vertex indices) are valid.
        const VALIDATED = 0x2,

        /// This flag is set by the validation postprocess-step (aiPostProcess_ValidateDS)
        /// if the validation is successful but some issues have been found.
        ///
        /// This can for example mean that a texture that does not exist is referenced
        /// by a material or that the bone weights for a vertex don't sum to 1.0 ... .
        /// In most cases you should still be able to use the import. This flag could
        /// be useful for applications which don't capture Assimp's log output.
        const VALIDATION_WARNING = 0x4,

        /// This flag is currently only set by the aiProcess_JoinIdenticalVertices step.
        /// It indicates that the vertices of the output meshes aren't in the internal
        /// verbose format anymore. In the verbose format all vertices are unique,
        /// no vertex is ever referenced by more than one face.
        const NON_VERBOSE_FORMAT = 0x8,

        /// Denotes pure height-map terrain data.
        ///
        /// Pure terrains usually consist of quads,
        /// sometimes triangles, in a regular grid. The x,y coordinates of all vertex
        /// positions refer to the x,y coordinates on the terrain height map, the z-axis
        /// stores the elevation at a specific point.
        ///
        /// TER (Terragen) and HMP (3D Game Studio) are height map formats.
        /// @note Assimp is probably not the best choice for loading *huge* terrains -
        /// fully triangulated data takes extremely much free store and should be avoided
        /// as long as possible (typically you'll do the triangulation when you actually
        /// need to render it).
        const TERRAIN = 0x10,
    }
}
ai_impl_enum!(SceneFlags, c_uint);

/// The root structure of the imported data.
///
/// Everything that was imported from the given file can be accessed from here.
/// Objects of this class are generally maintained and owned by Assimp, not
/// by the caller. You shouldn't want to instance it, nor should you ever try to
/// delete a given scene on your own.
pub struct Scene {
    raw: &'static ffi::aiScene,
}

impl Drop for Scene {
    fn drop(&mut self) {
        unsafe {
            ffi::aiReleaseImport(self.raw as *const _);
        }
    }
}

impl Scene {
    pub unsafe fn from_ptr(ptr: *const ffi::aiScene) -> Self {
        assert!(!ptr.is_null());
        Scene { raw: &*ptr }
    }

    fn get_error_string() -> String {
        unsafe {
            CStr::from_ptr(ffi::aiGetErrorString()).to_string_lossy().into_owned()
        }
    }

    /// TODO
    ///
    /// * return error (with log)
    /// * also: warnings?
    /// * doc
    /// * aiPropertyStore?
    #[allow(non_snake_case)]
    pub fn from_file(path: &str, flags: PostProcessSteps) -> Result<Scene, String> {
        let pFile = path.as_ptr() as *const _;
        let pFlags = flags.bits() as c_uint;
        unsafe {
            let ptr = ffi::aiImportFile(pFile, pFlags);
            if ptr.is_null() {
                return Err(Self::get_error_string())
            }
            Ok(Self::from_ptr(ptr))
        }
    }

    /// TODO return error (with log)
    ///
    /// * return error (with log)
    /// * also: warnings?
    /// * doc
    /// * aiPropertyStore?
    #[allow(non_snake_case)]
    pub fn from_bytes(bytes: &[u8], hint: &str, flags: PostProcessSteps) -> Result<Scene, String> {
        let pBuffer = bytes.as_ptr() as *const _;
        let pLength = bytes.len() as c_uint;
        let pFlags = flags.bits() as c_uint;
        let hint = format!("{}\0", hint);
        let pHint = hint.as_ptr() as *const _;
        unsafe {
            let ptr = ffi::aiImportFileFromMemory(pBuffer, pLength, pFlags, pHint);
            if ptr.is_null() {
                return Err(Self::get_error_string())
            }
            Ok(Self::from_ptr(ptr))
        }
    }

    /// Any combination of the AI_SCENE_FLAGS_XXX flags.
    ///
    /// By default
    /// this value is 0, no flags are set. Most applications will
    /// want to reject all scenes with the AI_SCENE_FLAGS_INCOMPLETE
    /// bit set.
    pub fn flags(&self) -> SceneFlags {
        unsafe { SceneFlags::from_ffi(self.raw.mFlags) }
    }

    /// The root node of the hierarchy.
    ///
    /// There will always be at least the root node if the import
    /// was successful (and no special flags have been set).
    /// Presence of further nodes depends on the format and content
    /// of the imported file.
    pub fn root_node(&self) -> Node {
        unsafe { Node::from_ptr(self.raw.mRootNode) }
    }

    /// The array of meshes.
    ///
    /// Use the indices given in the aiNode structure to access
    /// this array. The array is mNumMeshes in size. If the
    /// AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
    /// be at least ONE material.
    pub fn meshes(&self) -> &[Mesh] {
        unsafe { Mesh::slice(self.raw.mMeshes, self.raw.mNumMeshes) }
    }

    /// The array of materials.
    ///
    /// Use the index given in each aiMesh structure to access this
    /// array. The array is mNumMaterials in size. If the
    /// AI_SCENE_FLAGS_INCOMPLETE flag is not set there will always
    /// be at least ONE material.
    pub fn materials(&self) -> &[Material] {
        unsafe { Material::slice(self.raw.mMaterials, self.raw.mNumMaterials) }
    }

    /// The array of animations.
    ///
    /// All animations imported from the given file are listed here.
    /// The array is mNumAnimations in size.
    pub fn animations(&self) -> &[Animation] {
        unsafe { Animation::slice(self.raw.mAnimations, self.raw.mNumAnimations) }
    }

    /// The array of embedded textures.
    ///
    /// Not many file formats embed their textures into the file.
    /// An example is Quake's MDL format (which is also used by
    /// some GameStudio versions)
    pub fn textures(&self) -> &[Texture] {
        unsafe { Texture::slice(self.raw.mTextures, self.raw.mNumTextures) }
    }

    /// The array of light sources.
    ///
    /// All light sources imported from the given file are
    /// listed here. The array is mNumLights in size.
    pub fn lights(&self) -> &[Light] {
        unsafe { Light::slice(self.raw.mLights, self.raw.mNumLights) }
    }

    /// The array of cameras.
    ///
    /// All cameras imported from the given file are listed here.
    /// The array is mNumCameras in size. The first camera in the
    /// array (if existing) is the default camera view into
    /// the scene.
    pub fn cameras(&self) -> &[Camera] {
        unsafe { Camera::slice(self.raw.mCameras, self.raw.mNumCameras) }
    }
}
