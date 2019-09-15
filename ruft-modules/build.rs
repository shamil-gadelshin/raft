fn main() {
    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir("./src/communication/network/client_communicator/grpc/generated");

    tower_grpc_build::Config::from_prost(prost_config)
        .enable_server(true)
        .enable_client(true)
        .build(
            &["src/communication/network/client_communicator/grpc/proto/client_communicator.proto"],
            &["src/communication/network/client_communicator/grpc/proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
    println!("cargo:rerun-if-changed=communication/network/client_communicator/grpc/proto//client_communicator.proto");
}
