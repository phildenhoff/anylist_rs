pub mod api;
pub mod lists;
pub mod login;
pub mod recipes;

pub mod protobuf {
    pub mod anylist {
        include!(concat!(env!("OUT_DIR"), "/anylist.proto.rs"));
    }
}
