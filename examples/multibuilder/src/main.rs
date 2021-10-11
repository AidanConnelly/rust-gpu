use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() {
    let result = SpirvBuilder::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../shaders/compute-shader"),
        "spirv-unknown-spv1.3",
    )
    .print_metadata(MetadataPrintout::DependencyOnly)
    .multimodule(true)
    .build()
    .unwrap();
    println!("{:#?}", result);
}
