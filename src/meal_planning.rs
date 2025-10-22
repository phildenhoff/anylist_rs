use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::protobuf::anylist::{
    pb_operation_metadata::OperationClass, PbCalendarEvent, PbCalendarOperation,
    PbCalendarOperationList, PbOperationMetadata,
};
use chrono::NaiveDate;
use prost::Message;
use crate::utils::generate_id;

/// Represents a meal planning calendar event
#[derive(Debug, Clone)]
pub struct MealPlanEvent {
    pub id: String,
    pub date: String,
    pub title: Option<String>,
    pub recipe_id: Option<String>,
    pub label_id: Option<String>,
    pub details: Option<String>,
}

impl AnyListClient {
    /// Get meal plan events for a date range
    ///
    /// # Arguments
    ///
    /// * `start_date` - Start date in YYYY-MM-DD format
    /// * `end_date` - End date in YYYY-MM-DD format
    pub async fn get_meal_plan_events(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<MealPlanEvent>> {
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| AnyListError::Other(format!("Invalid start date: {}", e)))?;
        let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| AnyListError::Other(format!("Invalid end date: {}", e)))?;

        let data = self.get_user_data().await?;
        let events = match data.meal_planning_calendar_response {
            Some(ref res) => res
                .events
                .iter()
                .filter(|e| {
                    if let Some(date) = &e.date {
                        let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok();

                        if let Some(parsed_date) = parsed_date {
                            return parsed_date >= start && parsed_date <= end;
                        }
                        false
                    } else {
                        false
                    }
                })
                .map(|e| MealPlanEvent {
                    id: e.identifier.clone(),
                    date: e.date.clone().unwrap_or_default(),
                    title: e.title.clone(),
                    recipe_id: e.recipe_id.clone(),
                    label_id: e.label_id.clone(),
                    details: e.details.clone(),
                })
                .collect(),
            None => Vec::new(),
        };
        Ok(events)
    }

    /// Create a meal plan event
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the meal planning calendar
    /// * `date` - The date in YYYY-MM-DD format
    /// * `recipe_id` - Optional recipe ID
    /// * `title` - Optional title (for non-recipe events)
    /// * `label_id` - Optional meal label ID (Breakfast, Lunch, Dinner, etc.)
    pub async fn create_meal_plan_event(
        &self,
        calendar_id: &str,
        date: &str,
        recipe_id: Option<&str>,
        title: Option<&str>,
        label_id: Option<&str>,
    ) -> Result<MealPlanEvent> {
        let event_id = generate_id();
        let operation_id = generate_id();

        let new_event = PbCalendarEvent {
            identifier: event_id.clone(),
            logical_timestamp: Some(1),
            calendar_id: Some(calendar_id.to_string()),
            date: Some(date.to_string()),
            title: title.map(|t| t.to_string()),
            details: None,
            recipe_id: recipe_id.map(|r| r.to_string()),
            label_id: label_id.map(|l| l.to_string()),
            order_added_sort_index: Some(0),
            recipe_scale_factor: Some(1.0),
        };

        let operation = PbCalendarOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("create-meal-event".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            calendar_id: Some(calendar_id.to_string()),
            updated_event: Some(new_event),
            original_event: None,
            updated_label: None,
            original_label: None,
            sorted_label_ids: vec![],
            event_ids: vec![],
            updated_events: vec![],
            original_events: vec![],
        };

        let operation_list = PbCalendarOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("meal-planning/update", buf).await?;

        Ok(MealPlanEvent {
            id: event_id,
            date: date.to_string(),
            title: title.map(|t| t.to_string()),
            recipe_id: recipe_id.map(|r| r.to_string()),
            label_id: label_id.map(|l| l.to_string()),
            details: None,
        })
    }

    /// Update a meal plan event
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the meal planning calendar
    /// * `event_id` - The ID of the event to update
    /// * `date` - The new date in YYYY-MM-DD format
    /// * `recipe_id` - Optional new recipe ID
    /// * `title` - Optional new title
    /// * `label_id` - Optional new label ID
    pub async fn update_meal_plan_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        date: &str,
        recipe_id: Option<&str>,
        title: Option<&str>,
        label_id: Option<&str>,
    ) -> Result<()> {
        let operation_id = generate_id();

        let updated_event = PbCalendarEvent {
            identifier: event_id.to_string(),
            logical_timestamp: Some(1),
            calendar_id: Some(calendar_id.to_string()),
            date: Some(date.to_string()),
            title: title.map(|t| t.to_string()),
            details: None,
            recipe_id: recipe_id.map(|r| r.to_string()),
            label_id: label_id.map(|l| l.to_string()),
            order_added_sort_index: Some(0),
            recipe_scale_factor: Some(1.0),
        };

        let operation = PbCalendarOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("update-meal-event".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            calendar_id: Some(calendar_id.to_string()),
            updated_event: Some(updated_event),
            original_event: None,
            updated_label: None,
            original_label: None,
            sorted_label_ids: vec![],
            event_ids: vec![],
            updated_events: vec![],
            original_events: vec![],
        };

        let operation_list = PbCalendarOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("meal-planning/update", buf).await?;
        Ok(())
    }

    /// Delete a meal plan event
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the meal planning calendar
    /// * `event_id` - The ID of the event to delete
    pub async fn delete_meal_plan_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<()> {
        let operation_id = generate_id();

        let operation = PbCalendarOperation {
            metadata: Some(PbOperationMetadata {
                operation_id: Some(operation_id),
                handler_id: Some("delete-meal-event".to_string()),
                user_id: Some(self.user_id.clone()),
                operation_class: Some(OperationClass::UndefinedOperation as i32),
            }),
            calendar_id: Some(calendar_id.to_string()),
            updated_event: None,
            original_event: None,
            updated_label: None,
            original_label: None,
            sorted_label_ids: vec![],
            event_ids: vec![event_id.to_string()],
            updated_events: vec![],
            original_events: vec![],
        };

        let operation_list = PbCalendarOperationList {
            operations: vec![operation],
        };

        let mut buf = Vec::new();
        operation_list.encode(&mut buf).map_err(|e| {
            AnyListError::ProtobufError(format!("Failed to encode operation: {}", e))
        })?;

        self.post("meal-planning/update", buf).await?;
        Ok(())
    }

    /// Add ingredients from meal plan to a shopping list for a date range
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to add ingredients to
    /// * `start_date` - Start date in YYYY-MM-DD format
    /// * `end_date` - End date in YYYY-MM-DD format
    pub async fn add_meal_plan_ingredients_to_list(
        &self,
        list_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<()> {
        let events = self.get_meal_plan_events(start_date, end_date).await?;

        for event in events {
            if let Some(recipe_id) = event.recipe_id {
                // Add recipe ingredients to the list
                self.add_recipe_to_list(&recipe_id, list_id, None).await?;
            }
        }

        Ok(())
    }
}
