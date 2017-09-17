use prim::{self, Vector3};
use ffi;

#[derive(Debug, Clone, Copy)]
pub enum MetadataValue<'a> {
    Bool(bool),
    I32(i32),
    U64(u64),
    F32(f32),
    Vector3(Vector3),
    String(&'a str),
}

ai_ptr_type!{
    /// Container for holding metadata.
    ///
    /// Metadata is a key-value store using string keys and values.
    type MetaData: ffi::aiMetadata;
}

impl<'a> MetaData<'a> {
    pub fn iter(&self) -> Iter {
        Iter::new(self.raw())
    }
    pub fn get(&self, key: &str) -> Option<MetadataValue> {
        self.iter().find(|&(k, _)| k == key).map(|(_, v)| v)
    }
}

#[derive(Clone)]
pub struct Iter<'a> {
    raw: &'a ffi::aiMetadata,
    idx: usize,
}

impl<'a> Iter<'a> {
    fn new(raw: &'a ffi::aiMetadata) -> Self {
        assert!(!raw.mKeys.is_null() && !raw.mValues.is_null());
        Iter { raw: raw, idx: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, MetadataValue<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.raw.mNumProperties as usize {
            return None;
        }
        self.idx += 1;

        unsafe {
            use ffi::aiMetadataType::*;

            let key = prim::str(&*self.raw.mKeys.offset(self.idx as isize)).unwrap();
            let val_ptr = self.raw.mValues.offset(self.idx as isize);
            if val_ptr.is_null() {
                return self.next();
            }
            let val_raw = &*val_ptr;
            let val = match val_raw.mType {
                AI_BOOL => MetadataValue::Bool(*(val_raw.mData as *const bool)),
                AI_INT => MetadataValue::I32(*(val_raw.mData as *const i32)),
                AI_UINT64 => MetadataValue::U64(*(val_raw.mData as *const u64)),
                AI_FLOAT => MetadataValue::F32(*(val_raw.mData as *const f32)),
                AI_AIVECTOR3D => MetadataValue::Vector3(*(val_raw.mData as *const Vector3)),
                AI_AISTRING => MetadataValue::String(prim::str(&*(val_raw.mData as *const ffi::aiString)).unwrap()),
                _ => unreachable!(),
            };
            Some((key, val))
        }
    }
}
