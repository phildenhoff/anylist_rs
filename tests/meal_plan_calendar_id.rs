use anylist_rs::MealPlanEvent;

#[test]
fn meal_plan_event_exposes_calendar_id() {
    let event: MealPlanEvent = serde_json::from_value(serde_json::json!({
        "id": "event-1",
        "calendar_id": "calendar-1",
        "date": "2026-06-15",
        "title": "Dinner",
        "recipe_id": "recipe-1",
        "label_id": "label-1",
        "details": null
    }))
    .unwrap();

    assert_eq!(event.calendar_id(), Some("calendar-1"));
}

#[test]
fn meal_plan_event_without_calendar_id_deserializes_to_none() {
    let event: MealPlanEvent = serde_json::from_value(serde_json::json!({
        "id": "event-1",
        "date": "2026-06-15",
        "title": "Dinner",
        "recipe_id": null,
        "label_id": null,
        "details": null
    }))
    .unwrap();

    assert_eq!(event.calendar_id(), None);
}
