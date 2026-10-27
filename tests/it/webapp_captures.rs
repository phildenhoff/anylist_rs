//! Decodes every request the AnyList web client sent during the 2026-10-27
//! capture (`tests/fixtures/webapp_2026_10_27`, made with
//! `tools/webapp_capture`) with the crate's protobuf types.
//!
//! The snapshot lists, per request, the handler IDs and how many bytes prost
//! drops because `anylist.proto` lacks the field. Dropped bytes matter when we
//! send a stored message back: whatever prost dropped is missing from it.

use std::fmt::Write;
use std::path::Path;

use anylist_rs::protobuf::anylist::{
    PbCalendarOperationList, PbListFolderOperationList, PbListOperationList,
    PbListSettingsOperationList, PbMobileAppSettingsOperationList, PbRecipeOperationList,
    PbStarterListOperationList,
};
use prost::Message;

/// Decodes `bytes` as `M` and returns its handler IDs and the byte count lost
/// on a decode/encode round trip.
fn decode<M: Message + Default>(
    bytes: &[u8],
    handler_ids: impl Fn(&M) -> Vec<String>,
) -> (Vec<String>, usize) {
    let message = M::decode(bytes).expect("fixture decodes");
    (handler_ids(&message), bytes.len() - message.encoded_len())
}

macro_rules! handler_ids {
    ($list:expr) => {
        $list
            .operations
            .iter()
            .filter_map(|op| op.metadata.as_ref()?.handler_id.clone())
            .collect()
    };
}

#[test]
fn webapp_requests_2026_10_27_decode() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/webapp_2026_10_27");
    let mut table =
        String::from("scenario | request | endpoint | handler ids | bytes prost drops\n");

    for scenario in ["lists", "recipes", "meal-plan"] {
        let mut files: Vec<_> = std::fs::read_dir(root.join(scenario))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        files.sort();

        for path in files {
            let fixture: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let bytes = hex::decode(fixture["hex"].as_str().unwrap()).unwrap();
            let (ids, dropped) = match fixture["type"].as_str().unwrap() {
                "PBListOperationList" => decode(&bytes, |m: &PbListOperationList| handler_ids!(m)),
                "PBRecipeOperationList" => {
                    decode(&bytes, |m: &PbRecipeOperationList| handler_ids!(m))
                }
                "PBCalendarOperationList" => {
                    decode(&bytes, |m: &PbCalendarOperationList| handler_ids!(m))
                }
                "PBListFolderOperationList" => {
                    decode(&bytes, |m: &PbListFolderOperationList| handler_ids!(m))
                }
                "PBListSettingsOperationList" => {
                    decode(&bytes, |m: &PbListSettingsOperationList| handler_ids!(m))
                }
                "PBStarterListOperationList" => {
                    decode(&bytes, |m: &PbStarterListOperationList| handler_ids!(m))
                }
                "PBMobileAppSettingsOperationList" => {
                    decode(&bytes, |m: &PbMobileAppSettingsOperationList| {
                        handler_ids!(m)
                    })
                }
                other => panic!("{}: unexpected type {other}", path.display()),
            };

            let expected: Vec<String> =
                serde_json::from_value(fixture["handlerIds"].clone()).unwrap();
            assert_eq!(ids, expected, "{}", path.display());

            writeln!(
                table,
                "{scenario} | {} | {} | {} | {dropped}",
                path.file_stem().unwrap().to_string_lossy(),
                fixture["endpoint"].as_str().unwrap(),
                ids.join(", "),
            )
            .unwrap();
        }
    }

    insta::assert_snapshot!(table);
}
