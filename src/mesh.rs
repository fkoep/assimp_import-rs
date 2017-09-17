use prim::{self, Color4, Matrix4, Vector3};
use ffi;
use libc::c_uint;

pub type VertexIdx = c_uint;
pub type MaterialIdx = c_uint;

// ++++++++++++++++++++ Face ++++++++++++++++++++

ai_type!{
    /// A single face in a mesh, referring to multiple vertices.
    ///
    /// If mNumIndices is 3, we call the face 'triangle', for mNumIndices > 3
    /// it's called 'polygon' (hey, that's just a definition!).
    ///
    /// aiMesh::mPrimitiveTypes can be queried to quickly examine which prim of
    /// primitive are actually present in a mesh. The #aiProcess_SortByPType flag
    /// executes a special post-processing algorithm which splits meshes with
    /// *different* primitive prim mixed up (e.g. lines and triangles) in several
    /// 'clean' submeshes.
    ///
    /// Furthermore there is a configuration option (#AI_CONFIG_PP_SBP_REMOVE) to force
    /// #aiProcess_SortByPType to remove
    /// specific kinds of primitives from the imported scene, completely and forever.
    /// In many cases you'll probably want to set this setting to
    /// @code
    /// aiPrimitiveType_LINE|aiPrimitiveType_POINT
    /// @endcode
    /// Together with the #aiProcess_Triangulate flag you can then be sure that
    /// #aiFace::mNumIndices is always 3.
    /// @note Take a look at the @link data Data Structures page @endlink for
    /// more information on the layout and winding order of a face.
    type Face: ffi::aiFace;
}

impl Face {
    /// The indices defining this face.
    pub fn indices(&self) -> &[VertexIdx] {
        unsafe { prim::slice(self.raw.mIndices, self.raw.mNumIndices) }
    }
}

// ++++++++++++++++++++ VertexWeight ++++++++++++++++++++

ai_type!{
    /// A single influence of a bone on a vertex.
    #[derive(Clone, Copy)]
    type VertexWeight: ffi::aiVertexWeight;
}

impl VertexWeight {
    /// Index of the vertex which is influenced by the bone.
    pub fn vertex_idx(&self) -> VertexIdx {
        self.raw.mVertexId
    }

    /// The strength of the influence in the range (0...1).
    /// The influence from all bones at one vertex amounts to 1.
    pub fn weight(&self) -> f32 {
        self.raw.mWeight
    }
}

// ++++++++++++++++++++ Bone ++++++++++++++++++++

ai_ptr_type!{
    /// A single bone of a mesh.
    ///
    /// A bone has a name by which it can be found in the frame hierarchy and by
    /// which it can be addressed by animations. In addition it has a number of
    /// influences on vertices.
    type Bone: ffi::aiBone;
}

impl<'a> Bone<'a> {
    /// The name of the bone.
    pub fn name(&self) -> &str {
        prim::str(&self.raw().mName).unwrap()
    }

    /// The vertices affected by this bone
    pub fn weights(&self) -> &[VertexWeight] {
        unsafe { prim::slice(self.raw().mWeights, self.raw().mNumWeights) }
    }

    /// Matrix that transforms from mesh space to bone space in bind pose
    pub fn offset_matrix(&self) -> Matrix4 {
        prim::mat4(self.raw().mOffsetMatrix)
    }
}

// ++++++++++++++++++++ PrimitiveTypes ++++++++++++++++++++

bitflags!{
    /// Enumerates the prim of geometric primitives supported by Assimp.
    ///
    /// @see aiFace Face data structure
    /// @see aiProcess_SortByPType Per-primitive sorting of meshes
    /// @see aiProcess_Triangulate Automatic triangulation
    /// @see AI_CONFIG_PP_SBP_REMOVE Removal of specific primitive prim.
    pub flags PrimitiveTypes: c_uint {

        /// A point primitive.
        ///
        /// This is just a single vertex in the virtual world,
        /// #aiFace contains just one index for such a primitive.
        const POINT = 0x1,

        /// A line primitive.
        ///
        /// This is a line defined through a start and an end position.
        /// #aiFace contains exactly two indices for such a primitive.
        const LINE = 0x2,

        /// A triangular primitive.
        ///
        /// A triangle consists of three indices.
        const TRIANGLE = 0x4,

        /// A higher-level polygon with more than 3 edges.
        ///
        /// A triangle is a polygon, but polygon in this context means
        /// "all polygons that are not triangles". The "Triangulate"-Step
        /// is provided for your convenience, it splits all polygons in
        /// triangles (which are much easier to handle).
        const POLYGON = 0x8,
    }
}

ai_impl_enum!(PrimitiveTypes, c_uint);

// ++++++++++++++++++++ AnimMesh ++++++++++++++++++++
//
// TODO (not currently in use?)
//
// pub struct AnimMesh<'a> {
// raw: &'a ffi::aiAnimMesh
// }
//
// impl<'a> AnimMesh<'a> {
// pub fn num_vertices(&self) -> usize {
// self.raw.mNumVertices as usize
// }
//
// pub fn vertices(&self) -> &[Vector3] {
// unsafe {
// prim::vec3_slice(self.raw.mVertices, self.num_vertices())
// }
// }
// pub fn normals(&self) -> &[Vector3] {
// unsafe {
// prim::vec3_slice(self.raw.mNormals, self.num_vertices())
// }
// }
// pub fn tangents(&self) -> &[Vector3] {
// unsafe {
// prim::vec3_slice(self.raw.mTangents, self.num_vertices())
// }
// }
// pub fn bitangents(&self) -> &[Vector3] {
// unsafe {
// prim::vec3_slice(self.raw.mBitangents, self.num_vertices())
// }
// }
//
// TODO colors, coords
// }

// ++++++++++++++++++++ Mesh ++++++++++++++++++++

ai_ptr_type!{
    /// A mesh represents a geometry or model with a single material.
    ///
    /// It usually consists of a number of vertices and a series of primitives/faces
    /// referencing the vertices. In addition there might be a series of bones, each
    /// of them addressing a number of vertices with a certain weight. Vertex data
    /// is presented in channels with each channel containing a single per-vertex
    /// information such as a set of texture coords or a normal vector.
    /// If a data pointer is non-null, the corresponding data stream is present.
    /// From C++-programs you can also use the comfort functions Has*() to
    /// test for the presence of various data streams.
    ///
    /// A Mesh uses only a single material which is referenced by a material ID.
    /// @note The mPositions member is usually not optional. However, vertex positions
    /// *could* be missing if the #AI_SCENE_FLAGS_INCOMPLETE flag is set in
    /// @code
    /// aiScene::mFlags
    /// @endcode
    type Mesh: ffi::aiMesh;
}

impl<'a> Mesh<'a> {
    /// Name of the mesh. Meshes can be named, but this is not a
    /// requirement and leaving this field empty is totally fine.
    ///
    /// There are mainly three uses for mesh names:
    ///
    ///  - some formats name nodes and meshes independently.
    ///  - importers tend to split meshes up to meet the
    ///     one-material-per-mesh requirement. Assigning
    ///     the same (dummy) name to each of the result meshes
    ///     aids the caller at recovering the original mesh
    ///     partitioning.
    ///  - Vertex animations refer to meshes by their names.
    pub fn name(&self) -> Option<&str> {
        prim::str(&self.raw().mName)
    }

    /// Bitwise combination of the members of the #aiPrimitiveType enum.
    ///
    /// This specifies which prim of primitives are present in the mesh.
    /// The "SortByPrimitiveType"-Step can be used to make sure the
    /// output meshes consist of one primitive type each.
    pub fn primitive_types(&self) -> PrimitiveTypes {
        unsafe { PrimitiveTypes::from_ffi(self.raw().mPrimitiveTypes) }
    }

    /// Vertex positions.
    ///
    /// This array is always present in a mesh. The array is
    /// mNumVertices in size.
    pub fn vertices(&self) -> &[Vector3] {
        unsafe { prim::slice(self.raw().mVertices, self.raw().mNumVertices) }
    }

    /// Vertex normals.
    ///
    /// The array contains normalized vectors, NULL if not present.
    /// The array is mNumVertices in size. Normals are undefined for
    /// point and line primitives. A mesh consisting of points and
    /// lines only may not have normal vectors. Meshes with mixed
    /// primitive prim (i.e. lines and triangles) may have normals,
    /// but the normals for vertices that are only referenced by
    /// point or line primitives are undefined and set to QNaN (WARN:
    /// qNaN compares to inequal to *everything*, even to qNaN itself.
    /// Using code like this to check whether a field is qnan is:
    ///
    /// ```
    /// #define IS_QNAN(f) (f != f)
    /// ```
    ///
    /// still dangerous because even 1.f == 1.f could evaluate to false! (
    /// remember the subtleties of IEEE754 artithmetics). Use stuff like
    /// @c fpclassify instead.
    ///
    /// @note Normal vectors computed by Assimp are always unit-length.
    /// However, this needn't apply for normals that have been taken
    ///   directly from the model file.
    pub fn normals(&self) -> &[Vector3] {
        unsafe { prim::slice(self.raw().mNormals, self.raw().mNumVertices) }
    }

    /// Vertex tangents.
    ///
    /// The tangent of a vertex points in the direction of the positive
    /// X texture axis. The array contains normalized vectors, NULL if
    /// not present. The array is mNumVertices in size. A mesh consisting
    /// of points and lines only may not have normal vectors. Meshes with
    /// mixed primitive prim (i.e. lines and triangles) may have
    /// normals, but the normals for vertices that are only referenced by
    /// point or line primitives are undefined and set to qNaN.  See
    /// the #mNormals member for a detailed discussion of qNaNs.
    /// @note If the mesh contains tangents, it automatically also
    /// contains bitangents.
    pub fn tangents(&self) -> &[Vector3] {
        unsafe { prim::slice(self.raw().mTangents, self.raw().mNumVertices) }
    }

    /// Vertex bitangents.
    ///
    /// The bitangent of a vertex points in the direction of the positive
    /// Y texture axis. The array contains normalized vectors, NULL if not
    /// present. The array is mNumVertices in size.
    /// @note If the mesh contains tangents, it automatically also contains
    /// bitangents.
    pub fn bitangents(&self) -> &[Vector3] {
        unsafe { prim::slice(self.raw().mBitangents, self.raw().mNumVertices) }
    }

    /// Vertex color sets.
    ///
    /// A mesh may contain 0 to #AI_MAX_NUMBER_OF_COLOR_SETS vertex
    /// colors per vertex. NULL if not present. Each array is
    /// mNumVertices in size if present.
    pub fn colors(&self, channel: usize) -> &[Color4] {
        if channel >= ffi::AI_MAX_NUMBER_OF_COLOR_SETS {
            return &[];
        }
        unsafe { prim::slice(self.raw().mColors[channel], self.raw().mNumVertices) }
    }

    /// Vertex texture coords, also known as UV channels.
    ///
    /// A mesh may contain 0 to AI_MAX_NUMBER_OF_TEXTURECOORDS per
    /// vertex. NULL if not present. The array is mNumVertices in size.
    pub fn texture_coords(&self, channel: usize) -> &[Vector3] {
        if channel >= ffi::AI_MAX_NUMBER_OF_TEXTURECOORDS {
            return &[];
        }
        unsafe { prim::slice(self.raw().mTextureCoords[channel], self.raw().mNumVertices) }
    }

    /// Specifies the number of components for a given UV channel.
    ///
    /// Up to three channels are supported (UVW, for accessing volume
    /// or cube maps). If the value is 2 for a given channel n, the
    /// component p.z of mTextureCoords[n][p] is set to 0.0f.
    /// If the value is 1 for a given channel, p.y is set to 0.0f, too.
    /// @note 4D coords are not supported
    pub fn num_uv_components(&self, channel: usize) -> usize {
        if channel >= ffi::AI_MAX_NUMBER_OF_TEXTURECOORDS {
            return 0;
        }
        self.raw().mNumUVComponents[channel] as usize
    }

    /// The faces the mesh is constructed from.
    ///
    /// Each face refers to a number of vertices by their indices.
    /// This array is always present in a mesh, its size is given
    /// in mNumFaces. If the #AI_SCENE_FLAGS_NON_VERBOSE_FORMAT
    /// is NOT set each face references an unique set of vertices.
    pub fn faces(&self) -> &[Face] {
        unsafe { Face::slice(self.raw().mFaces, self.raw().mNumFaces) }
    }

    /// The bones of this mesh.
    ///
    /// A bone consists of a name by which it can be found in the
    /// frame hierarchy and a set of vertex weights.
    pub fn bones(&self) -> &[Bone] {
        unsafe { Bone::slice(self.raw().mBones, self.raw().mNumBones) }
    }

    /// The material used by this mesh.
    ///
    /// A mesh uses only a single material. If an imported model uses
    /// multiple materials, the import splits up the mesh. Use this value
    /// as index into the scene's material list.
    pub fn material_idx(&self) -> MaterialIdx {
        self.raw().mMaterialIndex
    }

    // TODO anim meshes (currently not in use?)
}
