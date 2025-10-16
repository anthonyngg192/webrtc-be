use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    prost_build::compile_protos(
        &[
            "src/proto/socket_event.proto",
            "src/proto/error_event.proto",
        ],
        &["src/"],
    )
    .expect("Failed to compile Protobuf files");

    Ok(())
}
