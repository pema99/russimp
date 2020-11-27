use russimp_sys::{
    aiScene,
    aiImportFile,
    aiReleaseImport,
    aiPostProcessSteps_aiProcess_CalcTangentSpace,
    aiPostProcessSteps_aiProcess_Triangulate,
    aiPostProcessSteps_aiProcess_JoinIdenticalVertices,
    aiPostProcessSteps_aiProcess_SortByPType,
    aiPostProcessSteps_aiProcess_MakeLeftHanded,
    aiPostProcessSteps_aiProcess_RemoveComponent,
    aiPostProcessSteps_aiProcess_GenNormals,
    aiPostProcessSteps_aiProcess_GenSmoothNormals,
    aiPostProcessSteps_aiProcess_SplitLargeMeshes,
    aiPostProcessSteps_aiProcess_PreTransformVertices,
    aiPostProcessSteps_aiProcess_LimitBoneWeights,
    aiPostProcessSteps_aiProcess_ValidateDataStructure,
    aiPostProcessSteps_aiProcess_ImproveCacheLocality,
    aiPostProcessSteps_aiProcess_RemoveRedundantMaterials,
    aiPostProcessSteps_aiProcess_FixInfacingNormals,
    aiPostProcessSteps_aiProcess_FindDegenerates,
    aiPostProcessSteps_aiProcess_FindInvalidData,
    aiPostProcessSteps_aiProcess_GenUVCoords,
    aiPostProcessSteps_aiProcess_TransformUVCoords,
    aiPostProcessSteps_aiProcess_FindInstances,
    aiPostProcessSteps_aiProcess_OptimizeMeshes,
    aiPostProcessSteps_aiProcess_OptimizeGraph,
    aiPostProcessSteps_aiProcess_FlipUVs,
    aiPostProcessSteps_aiProcess_FlipWindingOrder,
    aiPostProcessSteps_aiProcess_SplitByBoneCount,
    aiPostProcessSteps_aiProcess_Debone,
    aiPostProcessSteps_aiProcess_GlobalScale,
    aiPostProcessSteps_aiProcess_EmbedTextures,
    aiPostProcessSteps_aiProcess_ForceGenNormals,
    aiPostProcessSteps_aiProcess_DropNormals,
    aiPostProcessSteps_aiProcess_GenBoundingBoxes,
    aiGetErrorString,
    aiMaterial,
    aiAnimation,
    aiCamera,
    aiLight,
    aiMesh};

use std::{
    ffi::{
        CString,
        CStr,
    },
    ops::BitOr,
    ptr::slice_from_raw_parts,
};

use crate::{Russult, RussimpError, FromRawVec};
use crate::material::Material;
use crate::mesh::Mesh;
use std::os::raw::c_uint;

pub struct Scene<'a> {
    scene: *const aiScene,
    pub materials: Vec<Material<'a>>,
    pub meshes: Vec<Mesh<'a>>,
}

#[repr(u32)]
pub enum PostProcessSteps {
    CalcTangentSpace = aiPostProcessSteps_aiProcess_CalcTangentSpace,
    JoinIdenticalVertices = aiPostProcessSteps_aiProcess_JoinIdenticalVertices,
    MakeLeftHanded = aiPostProcessSteps_aiProcess_MakeLeftHanded,
    Triangulate = aiPostProcessSteps_aiProcess_Triangulate,
    RemoveComponent = aiPostProcessSteps_aiProcess_RemoveComponent,
    GenNormals = aiPostProcessSteps_aiProcess_GenNormals,
    GenSmoothNormals = aiPostProcessSteps_aiProcess_GenSmoothNormals,
    SplitLargeMeshes = aiPostProcessSteps_aiProcess_SplitLargeMeshes,
    PreTransformVertices = aiPostProcessSteps_aiProcess_PreTransformVertices,
    LimitBoneWeights = aiPostProcessSteps_aiProcess_LimitBoneWeights,
    ValidateDataStructure = aiPostProcessSteps_aiProcess_ValidateDataStructure,
    ImproveCacheLocality = aiPostProcessSteps_aiProcess_ImproveCacheLocality,
    RemoveRedundantMaterials = aiPostProcessSteps_aiProcess_RemoveRedundantMaterials,
    FixInfacingNormals = aiPostProcessSteps_aiProcess_FixInfacingNormals,
    SortByPType = aiPostProcessSteps_aiProcess_SortByPType,
    FindDegenerates = aiPostProcessSteps_aiProcess_FindDegenerates,
    FindInvalidData = aiPostProcessSteps_aiProcess_FindInvalidData,
    GenUVCoords = aiPostProcessSteps_aiProcess_GenUVCoords,
    TransformUVCoords = aiPostProcessSteps_aiProcess_TransformUVCoords,
    FindInstances = aiPostProcessSteps_aiProcess_FindInstances,
    OptimizeMeshes = aiPostProcessSteps_aiProcess_OptimizeMeshes,
    OptimizeGraph = aiPostProcessSteps_aiProcess_OptimizeGraph,
    FlipUVs = aiPostProcessSteps_aiProcess_FlipUVs,
    FlipWindingOrder = aiPostProcessSteps_aiProcess_FlipWindingOrder,
    SplitByBoneCount = aiPostProcessSteps_aiProcess_SplitByBoneCount,
    Debone = aiPostProcessSteps_aiProcess_Debone,
    GlobalScale = aiPostProcessSteps_aiProcess_GlobalScale,
    EmbedTextures = aiPostProcessSteps_aiProcess_EmbedTextures,
    ForceGenNormals = aiPostProcessSteps_aiProcess_ForceGenNormals,
    DropNormals = aiPostProcessSteps_aiProcess_DropNormals,
    GenBoundingBoxes = aiPostProcessSteps_aiProcess_GenBoundingBoxes,
}

impl<'a> Drop for Scene<'a> {
    fn drop(&mut self) {
        unsafe {
            aiReleaseImport(self.scene);
        }
    }
}

impl<'a> FromRawVec for Scene<'a> {}

impl<'a> Scene<'a> {
    pub fn from(file_path: &str, flags: Vec<PostProcessSteps>) -> Russult<Scene<'a>> {
        let bitwise_flag = flags.into_iter().fold(0, |acc, x| acc | (x as u32));
        let c_str = CString::new(file_path).unwrap();
        let scene_import: *const aiScene = unsafe { aiImportFile(c_str.as_ptr(), bitwise_flag) };

        if scene_import.is_null() {
            let error_buf = unsafe { aiGetErrorString() };
            let error = unsafe { CStr::from_ptr(error_buf).to_string_lossy().into_owned() };
            return Err(RussimpError::Import(error));
        }

        Ok(Self {
            scene: scene_import,
            materials: Scene::get_vec_from_raw(unsafe { (*scene_import).mMaterials }, unsafe { (*scene_import).mNumMaterials }),
            meshes: Scene::get_vec_from_raw(unsafe { (*scene_import).mMeshes }, unsafe { (*scene_import).mNumMeshes }),
        })
    }
}

#[test]
fn importing_invalid_file_returns_error() {
    let current_directory_buf = std::env::current_dir().unwrap().join("russimp-sys/assimp/test/models/box.blend");

    let scene = Scene::from(current_directory_buf.to_str().unwrap(),
                            vec![PostProcessSteps::CalcTangentSpace,
                                 PostProcessSteps::Triangulate,
                                 PostProcessSteps::JoinIdenticalVertices,
                                 PostProcessSteps::SortByPType]);

    assert!(scene.is_err())
}

#[test]
fn importing_valid_file_returns_scene() {
    let current_directory_buf = std::env::current_dir().unwrap().join("russimp-sys/assimp/test/models/BLEND/box.blend");

    Scene::from(current_directory_buf.to_str().unwrap(),
                vec![PostProcessSteps::CalcTangentSpace,
                     PostProcessSteps::Triangulate,
                     PostProcessSteps::JoinIdenticalVertices,
                     PostProcessSteps::SortByPType]).unwrap();
}