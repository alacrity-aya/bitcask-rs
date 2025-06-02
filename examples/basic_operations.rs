use bitcast_rs::options::IndexType;
use std::path::PathBuf;

use bitcast_rs::{db, options::Options};
use bytes::Bytes;

fn main() {
    let opts = Options {
        dir_path: PathBuf::from("./data/"),
        data_file_size: 256 * 1024 * 1024,
        sync_writes: false,
        index_type: IndexType::Btree,
    };
    let engine = db::Engine::open(opts).expect("failed to open bitcast engine");

    let res1 = engine.put(Bytes::from("name"), Bytes::from("bitcast-rs"));
    assert!(res1.is_ok());

    let res2 = engine.get(Bytes::from("name"));
    assert!(res2.is_ok());

    let val = res2.ok().unwrap();
    println!("val = {:?}", String::from_utf8(val.to_vec()));
}
