fn main() {
    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir("./src/gprc_client_communicator/grpc");

    tower_grpc_build::Config::from_prost(prost_config)
        .enable_server(true)
        .enable_client(true)
        .build(
            &["src/gprc_client_communicator/proto/client_communicator.proto"],
            &["src/gprc_client_communicator/proto"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
    println!("cargo:rerun-if-changed=gprc_client_communicator/proto/client_communicator.proto");
}
