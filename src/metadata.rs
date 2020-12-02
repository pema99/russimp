use russimp_sys::{aiMetadata, aiMetadataType_AI_AISTRING, aiMetadataType_AI_AIVECTOR3D, aiMetadataType_AI_BOOL, aiMetadataType_AI_FLOAT, aiMetadataType_AI_DOUBLE, aiMetadataType_AI_INT32, aiMetadataType_AI_UINT64, aiMetadataType_AI_META_MAX, aiMetadataType_FORCE_32BIT, aiMetadataEntry, aiVector3D};

use crate::{FromRaw,
            scene::{PostProcessSteps, Scene},
            Russult,
            RussimpError};

use std::{
    any::Any,
    ffi::CStr,
    os::raw::c_char,
    borrow::Borrow,
};
use std::ptr::slice_from_raw_parts;

pub trait MetaDataEntryCast<'a> {
    fn can_cast(&self) -> bool;
    fn cast(&self) -> Russult<MetadataType<'a>>;
}

struct MetaDataEntryString<'a> {
    data: &'a aiMetadataEntry,
}

impl<'a> MetaDataEntryCast<'a> for MetaDataEntryString<'a> {
    fn can_cast(&self) -> bool {
        (self.data.mType & aiMetadataType_AI_AISTRING) != 0
    }

    fn cast(&self) -> Russult<MetadataType<'a>> {
        let cstr = unsafe { CStr::from_ptr(self.data.mData as *const c_char) };
        cstr.to_str().map_or_else(|e| Err(e.into()), |r| Ok(MetadataType::String(r.to_string())))
    }
}

struct MetaDataVector3d<'a> {
    data: &'a aiMetadataEntry,
}

impl<'a> MetaDataEntryCast<'a> for MetaDataVector3d<'a> {
    fn can_cast(&self) -> bool {
        (self.data.mType & aiMetadataType_AI_AIVECTOR3D) != 0
    }

    fn cast(&self) -> Russult<MetadataType<'a>> {
        let vec = self.data.mData as *mut aiVector3D;
        if let Some(content) = unsafe { vec.as_ref() } {
            return Ok(MetadataType::Vector3d(content));
        }

        Err(RussimpError::MetadataError("data is null".to_string()))
    }
}

pub struct MetaData<'a> {
    meta_data: &'a aiMetadata,
    pub keys: Vec<String>,
    pub values: Vec<MetaDataEntry<'a>>,
}

#[repr(u32)]
pub enum MetadataType<'a> {
    String(String),
    Vector3d(&'a aiVector3D),
    // Bool = aiMetadataType_AI_BOOL,
    // Float = aiMetadataType_AI_FLOAT,
    // Double = aiMetadataType_AI_DOUBLE,
    // Int = aiMetadataType_AI_INT32,
    // Long = aiMetadataType_AI_UINT64,
    // MetaMax = aiMetadataType_AI_META_MAX,
    // Force32 = aiMetadataType_FORCE_32BIT,
}

pub struct MetaDataEntry<'a> {
    raw: &'a aiMetadataEntry,
    pub data: Russult<MetadataType<'a>>,
}

impl<'a> MetaDataEntry<'a> {
    fn cast_data(data: &'a aiMetadataEntry) -> Russult<MetadataType<'a>> {
        let casters: Vec<Box<dyn MetaDataEntryCast<'a>>> = vec![Box::new(MetaDataVector3d {
            data
        }), Box::new(MetaDataEntryString {
            data
        })];

        for caster in casters {
            if caster.can_cast() {
                return caster.cast();
            }
        }

        Err(RussimpError::MetadataError("could not find caster for metadata type".to_string()))
    }
}

impl<'a> Into<MetaDataEntry<'a>> for &'a aiMetadataEntry {
    fn into(self) -> MetaDataEntry<'a> {
        MetaDataEntry {
            raw: self,
            data: MetaDataEntry::cast_data(self),
        }
    }
}

impl<'a> FromRaw for MetaData<'a> {}

impl<'a> Into<MetaData<'a>> for &'a aiMetadata {
    fn into(self) -> MetaData<'a> {
        MetaData {
            meta_data: self,
            keys: MetaData::get_vec(self.mKeys, self.mNumProperties),
            values: MetaData::get_vec(self.mValues, self.mNumProperties),
        }
    }
}

#[test]
fn metadata_for_box() {
    let current_directory_buf = std::env::current_dir().unwrap().join("russimp-sys/assimp/test/models/BLEND/box.blend");

    let scene = Scene::from(current_directory_buf.to_str().unwrap(),
                            vec![PostProcessSteps::CalcTangentSpace,
                                 PostProcessSteps::Triangulate,
                                 PostProcessSteps::JoinIdenticalVertices,
                                 PostProcessSteps::SortByPType]).unwrap();

    assert!(scene.metadata.is_none());
}