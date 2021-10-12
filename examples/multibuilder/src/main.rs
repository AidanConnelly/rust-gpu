use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() {
    let result = SpirvBuilder::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../shaders/compute-shader"),
        "spirv-unknown-spv1.3",
    )
        .capability(Capability::Int8)
        .print_metadata(MetadataPrintout::DependencyOnly)
        .multimodule(true)
        .build()
        .unwrap();
    println!("{:#?}", result);
}
