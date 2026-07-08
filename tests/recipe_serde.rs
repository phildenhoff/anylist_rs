use anylist_rs::Recipe;

/// JSON produced by anylist_rs before `ingredient_entries` existed (< 0.5)
/// must keep deserializing: the field defaults to an empty list.
#[test]
fn recipe_json_without_ingredient_entries_deserializes() {
    let old_json = r#"{
        "id": "r1",
        "name": "Bread",
        "ingredients": [
            {"name": "Flour", "quantity": "2 cups", "note": null, "raw_ingredient": null},
            {"name": "Water", "quantity": null, "note": null, "raw_ingredient": null}
        ],
        "preparation_steps": ["Mix", "Bake"],
        "note": null,
        "source_name": null,
        "source_url": null,
        "servings": null,
        "prep_time": null,
        "cook_time": null,
        "rating": null,
        "nutritional_info": null,
        "photo_id": null,
        "photo_urls": []
    }"#;

    let recipe: Recipe = serde_json::from_str(old_json).expect("old Recipe JSON must deserialize");
    assert_eq!(recipe.name(), "Bread");
    assert_eq!(recipe.ingredients().len(), 2);
    assert!(recipe.ingredient_entries().is_empty());
}

/// A serialize → deserialize round-trip through the current format preserves
/// the recipe, including its ingredient entries.
#[test]
fn recipe_json_round_trips() {
    let old_json = r#"{
        "id": "r1",
        "name": "Bread",
        "ingredients": [
            {"name": "Flour", "quantity": "2 cups", "note": null, "raw_ingredient": null}
        ],
        "preparation_steps": [],
        "note": null,
        "source_name": null,
        "source_url": null,
        "servings": null,
        "prep_time": null,
        "cook_time": null,
        "rating": null,
        "nutritional_info": null,
        "photo_id": null,
        "photo_urls": []
    }"#;

    let recipe: Recipe = serde_json::from_str(old_json).unwrap();
    let serialized = serde_json::to_string(&recipe).unwrap();
    let reparsed: Recipe = serde_json::from_str(&serialized).unwrap();
    assert_eq!(recipe, reparsed);
}
