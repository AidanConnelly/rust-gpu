use pretty_assertions::assert_eq;
use rspirv::binary::Disassemble;
use std::fs::{remove_file, File};
use std::io::prelude::*;
use std::process::Command;
use tempfile::tempdir;

// https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);
/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

const PREFIX: &str = r#"
#![feature(no_core, lang_items)]
#![no_core]

#[lang = "sized"]
pub trait Sized {}

#[lang = "unsize"]
pub trait Unsize<T: ?Sized> {}

#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T> {}

#[lang = "copy"]
pub unsafe trait Copy {}

unsafe impl Copy for bool {}
unsafe impl Copy for u8 {}
unsafe impl Copy for u16 {}
unsafe impl Copy for u32 {}
unsafe impl Copy for u64 {}
unsafe impl Copy for usize {}
unsafe impl Copy for i8 {}
unsafe impl Copy for i16 {}
unsafe impl Copy for i32 {}
unsafe impl Copy for isize {}
unsafe impl Copy for f32 {}
unsafe impl Copy for char {}
unsafe impl<'a, T: ?Sized> Copy for &'a T {}
unsafe impl<T: ?Sized> Copy for *const T {}
unsafe impl<T: ?Sized> Copy for *mut T {}

#[lang = "add"]
pub trait Add<RHS = Self> {
    type Output;

    fn add(self, rhs: RHS) -> Self::Output;
}

impl Add for u32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}
"#;

fn go(code: &str, expected: &str) {
    let temp = tempdir().expect("Unable to create temp dir");
    let input = temp.path().join("code.rs");
    let output = temp.path().join("code.spv");

    let mut input_data = PREFIX.to_string();
    input_data.push_str(code);
    File::create(&input)
        .unwrap()
        .write_all(&input_data.into_bytes())
        .unwrap();

    let cmd = Command::new("rustc")
        .args(&[
            #[cfg(target_os = "unix")]
            "-Zcodegen-backend=target/debug/librustc_codegen_spirv.so",
            #[cfg(target_os = "windows")]
            "-Zcodegen-backend=target/debug/rustc_codegen_spirv.dll",
            "--crate-type",
            "lib",
            "-O",
            "-Zmir-opt-level=3",
            "--out-dir",
        ])
        .arg(temp.path())
        .arg(&input)
        .status()
        .expect("failed to execute process");
    assert!(cmd.success());

    let mut output_data = Vec::new();
    File::open(&output)
        .unwrap()
        .read_to_end(&mut output_data)
        .unwrap();

    let output_disas = rspirv::dr::load_bytes(&output_data)
        .expect("failed to parse spirv")
        .disassemble();

    assert_eq!(PrettyString(&output_disas), PrettyString(expected));

    match Command::new("spirv-val").arg(&output).status() {
        Ok(status) => assert!(status.success()),
        Err(err) => eprint!("spirv-val tool not found, ignoring test: {}", err),
    }

    remove_file(input).expect("Failed to delete input file");
    remove_file(output).expect("Failed to delete output file");
    temp.close().expect("Failed to delete temp dir");
}

#[test]
pub fn it_works() {
    go(
        r"
pub fn add_numbers(x: u32, y: u32) -> u32 {
    x + y
}",
        r"; SPIR-V
; Version: 1.5
; Generator: rspirv
; Bound: 13
OpCapability Shader
OpCapability Linkage
OpMemoryModel Logical GLSL450
%1 = OpTypeInt 32 0
%2 = OpTypeFunction %1 %1 %1
%3 = OpFunction  %1  None %2
%4 = OpFunctionParameter  %1
%5 = OpFunctionParameter  %1
%6 = OpLabel
%7 = OpIAdd  %1  %4 %5
OpReturnValue %7
OpFunctionEnd
%8 = OpFunction  %1  None %2
%9 = OpFunctionParameter  %1
%10 = OpFunctionParameter  %1
%11 = OpLabel
%12 = OpIAdd  %1  %9 %10
OpReturnValue %12
OpFunctionEnd",
    );
}

#[test]
pub fn fib() {
    go(
        r"
pub fn fib(n: u32) -> u32 {
    let mut x = (1, 1);
    for _ in 1..n {
        x = (x.1, x.0 + x.1)
    }
    x.1
}",
        r"",
    );
}