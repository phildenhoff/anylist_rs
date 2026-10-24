use anylist_rs::protobuf::anylist::PbIngredient;
use prost::Message;

#[test]
fn ingredient_section_heading_round_trips_through_protobuf() {
    let ingredient = PbIngredient {
        raw_ingredient: None,
        name: Some("Sauce".to_string()),
        quantity: None,
        note: None,
        identifier: Some("heading-id".to_string()),
        is_heading: Some(true),
    };

    let mut bytes = Vec::new();
    ingredient.encode(&mut bytes).unwrap();
    let decoded = PbIngredient::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.is_heading, Some(true));
    assert_eq!(decoded.name.as_deref(), Some("Sauce"));
    assert_eq!(decoded.raw_ingredient, None);
}
