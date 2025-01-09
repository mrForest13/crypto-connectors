extern crate prost_build;

fn main() {
    prost_build::compile_protos(
        &[
            "proto/error.proto",
            "proto/market.proto",
            "proto/book.proto",
            "proto/ticker.proto",
            "proto/trade.proto",
            "proto/types.proto",
        ],
        &["proto/"],
    )
    .expect("protoc/Cargo.toml failed to compile protos");
}
