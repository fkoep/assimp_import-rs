use prim::{self, Quaternion, Vector3};
use ffi;

// ++++++++++++++++++++ key prim ++++++++++++++++++++

ai_type!{
    /// A time-value pair specifying a certain 3D vector for the given time.
    #[derive(Clone, Copy)]
    type VectorKey: ffi::aiVectorKey;
}

impl VectorKey {
    /// The time of this key
    pub fn time(&self) -> f64 {
        self.raw.mTime
    }
    /// The value of this key
    pub fn value(&self) -> Vector3 {
        prim::vec3(self.raw.mValue)
    }
}

ai_type!{
    /// A time-value pair specifying a rotation for the given time.
    /// Rotations are expressed with quaternions.
    #[derive(Clone, Copy)]
    type QuatKey: ffi::aiQuatKey;
}

impl QuatKey {
    /// The time of this key
    pub fn time(&self) -> f64 {
        self.raw.mTime
    }
    /// The value of this key
    pub fn value(&self) -> Quaternion {
        prim::quat(self.raw.mValue)
    }
}

// TODO mesh key, see mesh.rs

// ++++++++++++++++++++ AnimBehavior ++++++++++++++++++++

/// Defines how an animation channel behaves outside the defined time
/// range.
///
/// This corresponds to aiNodeAnim::mPreState and aiNodeAnim::mPostState.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum AnimBehavior {
    /// The value from the default node transformation is taken
    Default = 0x0,

    /// The nearest key value is used without interpolation
    Constant = 0x1,

    /// The value of the nearest two keys is linearly
    /// extrapolated for the current time value.
    Linear = 0x2,

    /// The animation is repeated.
    ///
    /// If the animation key go from n to m and the current
    /// time is t, use the value at (t-n) % (|m-n|).
    Repeat = 0x3,
}

ai_impl_enum!(AnimBehavior, ffi::aiAnimBehaviour);

// ++++++++++++++++++++ NodeAnim ++++++++++++++++++++

ai_ptr_type!{
    /// Describes the animation of a single node.
    ///
    /// The name specifies the
    /// bone/node which is affected by this animation channel. The keyframes
    /// are given in three separate series of values, one each for position,
    /// rotation and scaling. The transformation matrix computed from these
    /// values replaces the node's original transformation matrix at a
    /// specific time.
    /// This means all keys are absolute and not relative to the bone default pose.
    /// The order in which the transformations are applied is
    /// - as usual - scaling, rotation, translation.
    ///
    /// @note All keys are returned in their correct, chronological order.
    /// Duplicate keys don't pass the validation step. Most likely there
    /// will be no negative time values, but they are not forbidden also ( so
    /// implementations need to cope with them! )
    type NodeAnim: ffi::aiNodeAnim;
}

impl<'a> NodeAnim<'a> {
    /// The name of the node affected by this animation. The node
    /// must exist and it must be unique.
    pub fn node_name(&self) -> &str {
        prim::str(&self.raw().mNodeName).unwrap()
    }

    /// The position keys of this animation channel. Positions are
    /// specified as 3D vector. The array is mNumPositionKeys in size.
    ///
    /// If there are position keys, there will also be at least one
    /// scaling and one rotation key.
    pub fn position_keys(&self) -> &[VectorKey] {
        unsafe { VectorKey::slice(self.raw().mPositionKeys, self.raw().mNumPositionKeys) }
    }

    /// The rotation keys of this animation channel. Rotations are
    /// given as quaternions,  which are 4D vectors. The array is
    /// mNumRotationKeys in size.
    ///
    /// If there are rotation keys, there will also be at least one
    /// scaling and one position key.
    pub fn rotation_keys(&self) -> &[QuatKey] {
        unsafe { QuatKey::slice(self.raw().mRotationKeys, self.raw().mNumRotationKeys) }
    }

    /// The scaling keys of this animation channel. Scalings are
    /// specified as 3D vector. The array is mNumScalingKeys in size.
    ///
    /// If there are scaling keys, there will also be at least one
    /// position and one rotation key.
    pub fn scaling_keys(&self) -> &[VectorKey] {
        unsafe { VectorKey::slice(self.raw().mScalingKeys, self.raw().mNumScalingKeys) }
    }

    /// Defines how the animation behaves before the first
    /// key is encountered.
    ///
    /// The default value is aiAnimBehaviour_DEFAULT (the original
    /// transformation matrix of the affected node is used).
    pub fn pre_state(&self) -> AnimBehavior {
        unsafe { AnimBehavior::from_ffi(self.raw().mPreState) }
    }

    /// Defines how the animation behaves after the last
    /// key was processed.
    ///
    /// The default value is aiAnimBehaviour_DEFAULT (the original
    /// transformation matrix of the affected node is taken).
    pub fn post_state(&self) -> AnimBehavior {
        unsafe { AnimBehavior::from_ffi(self.raw().mPostState) }
    }
}

// ++++++++++++++++++++ MeshAnim ++++++++++++++++++++

// TODO? see mesh.rs

// ++++++++++++++++++++ Animation ++++++++++++++++++++

ai_ptr_type!{
    /// An animation consists of keyframe data for a number of nodes. For
    /// each node affected by the animation a separate series of data is given.
    type Animation: ffi::aiAnimation;
}

impl<'a> Animation<'a> {
    /// The name of the animation. If the modeling package this data was
    /// exported from does support only a single animation channel, this
    /// name is usually empty (length is zero).
    pub fn name(&self) -> Option<&str> {
        prim::str(&self.raw().mName)
    }

    /// Duration of the animation in ticks.
    pub fn duration(&self) -> f64 {
        self.raw().mDuration
    }

    /// Ticks per second. 0 if not specified in the imported file
    pub fn ticks_per_second(&self) -> f64 {
        self.raw().mTicksPerSecond
    }

    /// The node animation channels. Each channel affects a single node.
    /// The array is mNumChannels in size.
    pub fn channels(&self) -> &[NodeAnim] {
        unsafe { NodeAnim::slice(self.raw().mChannels, self.raw().mNumChannels) }
    }

    // TODO mesh_channels, see mesh.rs
}
