fn main() {
    let proto_root = "protos";
    println!("cargo:rerun-if-changed={}", proto_root);

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["protos/preconf.proto"], &[proto_root])
        .expect("failed to compile preconf proto");
}
