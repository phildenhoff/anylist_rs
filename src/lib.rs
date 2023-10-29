pub mod lists;
pub mod login;

// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod protobuf {
    pub mod anylist {
        include!(concat!(env!("OUT_DIR"), "/anylist.proto.rs"));
    }
}
