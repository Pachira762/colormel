use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use windows::{core::PCWSTR, Win32::Graphics::Direct3D::Dxc::*};

struct CompileTarget {
    pub file: String,
    pub entry: String,
    pub profile: String,
    pub defines: Vec<(String, String)>,
}

impl CompileTarget {
    fn new(file: &str, entry: &str) -> Self {
        let profile = if entry.ends_with("Vs") {
            "vs_6_6"
        } else if entry.ends_with("Ps") {
            "ps_6_6"
        } else if entry.ends_with("As") {
            "as_6_6"
        } else if entry.ends_with("Ms") {
            "ms_6_6"
        } else if entry.ends_with("Cs") {
            "cs_6_6"
        } else {
            unreachable!("")
        };

        let defines = if profile == "cs_6_6" {
            [("COMPUTE".to_string(), "".to_string())]
        } else {
            [("GRAPHICS".to_string(), "".to_string())]
        };

        Self {
            file: file.to_string(),
            entry: entry.to_string(),
            profile: profile.to_string(),
            defines: defines.to_vec(),
        }
    }

    fn in_path(&self) -> PathBuf {
        Path::new("src/shaders").join(&self.file)
    }

    fn out_path(&self) -> PathBuf {
        Path::new("src/shaders/bin").join(&(self.entry.clone() + ".bin"))
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-canged=build.rs");
    println!("cargo:rerun-if-canged=manifest.manifest");
    println!("cargo:rerun-if-canged=icon.ico");
    println!("cargo:rerun-if-canged=src/shaders");

    winres::WindowsResource::new()
        .set_manifest_file("manifest.manifest")
        .set_icon("icon.ico")
        .compile()?;

    Compiler::new()?
        .compile(&CompileTarget::new("colorcloud.hlsl", "ColorCloudCs"))?
        .compile(&CompileTarget::new("colorcloud.hlsl", "ColorCloudAs"))?
        .compile(&CompileTarget::new("colorcloud.hlsl", "ColorCloudMs"))?
        .compile(&CompileTarget::new("colorcloud.hlsl", "ColorCloudPs"))?
        .compile(&CompileTarget::new("filter.hlsl", "FilterVs"))?
        .compile(&CompileTarget::new("filter.hlsl", "FilterPs"))?
        .compile(&CompileTarget::new("histogram.hlsl", "HistogramCs"))?
        .compile(&CompileTarget::new("histogram.hlsl", "HistogramVs"))?
        .compile(&CompileTarget::new("histogram.hlsl", "HistogramPs"))?
        .compile(&CompileTarget::new("primitive.hlsl", "PrimitiveVs"))?
        .compile(&CompileTarget::new("primitive.hlsl", "PrimitivePs"))?;

    Ok(())
}

struct Compiler {
    util: IDxcUtils,
    compiler: IDxcCompiler3,
    include_handler: IDxcIncludeHandler,
}

impl Compiler {
    fn new() -> Result<Self> {
        unsafe {
            let util: IDxcUtils = DxcCreateInstance(&CLSID_DxcLibrary)?;
            let compiler = DxcCreateInstance(&CLSID_DxcCompiler)?;
            let include_handler = util.CreateDefaultIncludeHandler()?;

            Ok(Self {
                util,
                compiler,
                include_handler,
            })
        }
    }

    fn compile(&self, target: &CompileTarget) -> Result<&Self> {
        let path_buf = path_to_cstr(&target.in_path());
        let path = PCWSTR::from_raw(path_buf.as_ptr());

        let entry_path = str_to_cstr(&target.entry);
        let entry = PCWSTR::from_raw(entry_path.as_ptr());

        let profile_path = str_to_cstr(&target.profile);
        let profile = PCWSTR::from_raw(profile_path.as_ptr());

        let defines_buf: Vec<_> = target
            .defines
            .iter()
            .map(|(name, value)| (str_to_cstr(name), str_to_cstr(value)))
            .collect();
        let defines: Vec<_> = defines_buf
            .iter()
            .map(|(name, value)| DxcDefine {
                Name: PCWSTR::from_raw(name.as_ptr()),
                Value: PCWSTR::from_raw(value.as_ptr()),
            })
            .collect();

        let blob = self.compile_internal(path, entry, profile, &defines)?;
        let out_path = target.out_path();
        self.save(&out_path, blob)?;

        Ok(self)
    }

    fn compile_internal(
        &self,
        path: PCWSTR,
        entry: PCWSTR,
        profile: PCWSTR,
        defines: &[DxcDefine],
    ) -> Result<IDxcBlob> {
        unsafe {
            let args: IDxcCompilerArgs = self
                .util
                .BuildArguments(path, entry, profile, None, defines)?;

            let source = self.util.LoadFile(path, None)?;
            let result: IDxcResult = self.compiler.Compile(
                &DxcBuffer {
                    Ptr: source.GetBufferPointer(),
                    Size: source.GetBufferSize(),
                    Encoding: DXC_CP_ACP.0,
                },
                Some(std::slice::from_raw_parts(
                    args.GetArguments(),
                    args.GetCount() as _,
                )),
                &self.include_handler,
            )?;

            if let Err(e) = result.GetStatus()?.ok() {
                let mut error: Option<IDxcBlobUtf8> = None;
                let mut name = None;
                result.GetOutput(DXC_OUT_ERRORS, &mut name, &mut error)?;

                if let Some(error) = error {
                    let msg = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        error.GetBufferPointer() as _,
                        error.GetBufferSize(),
                    ));
                    anyhow::bail!(msg);
                }

                anyhow::bail!(e);
            }

            let mut blob: Option<IDxcBlob> = None;
            let mut name = None;
            result.GetOutput(DXC_OUT_OBJECT, &mut name, &mut blob)?;

            Ok(blob.unwrap())
        }
    }

    fn save(&self, path: &Path, blob: IDxcBlob) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        unsafe {
            file.write_all(std::slice::from_raw_parts(
                blob.GetBufferPointer() as _,
                blob.GetBufferSize() as _,
            ))?;
        }
        Ok(())
    }
}

fn str_to_cstr(str: &str) -> Vec<u16> {
    let mut wcs: Vec<_> = str.encode_utf16().collect();
    wcs.push(0);
    wcs
}

fn path_to_cstr(path: &Path) -> Vec<u16> {
    str_to_cstr(&path.to_string_lossy())
}
