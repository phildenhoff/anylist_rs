use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::operations;
use crate::protobuf::anylist::{pb_starter_list, PbCalendarEvent, PbListItem, PbUserDataResponse};
use chrono::NaiveDate;
use serde_derive::{Deserialize, Serialize};

/// Represents a meal planning calendar event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealPlanEvent {
    id: String,
    calendar_id: Option<String>,
    date: String,
    title: Option<String>,
    recipe_id: Option<String>,
    label_id: Option<String>,
    details: Option<String>,
}

impl MealPlanEvent {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn calendar_id(&self) -> Option<&str> {
        self.calendar_id.as_deref()
    }

    /// Get the date in YYYY-MM-DD format
    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn recipe_id(&self) -> Option<&str> {
        self.recipe_id.as_deref()
    }

    pub fn label_id(&self) -> Option<&str> {
        self.label_id.as_deref()
    }

    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }
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
                .map(meal_plan_event_from_pb)
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
        let data = self.get_user_data().await?;
        let event = PbCalendarEvent {
            date: Some(date.to_string()),
            title: title.map(str::to_string),
            recipe_id: recipe_id.map(str::to_string),
            label_id: label_id.map(str::to_string),
            ..Default::default()
        };
        let (event, requests) =
            operations::create_event(&mut self.ids(), calendar_id, event, calendar_events(&data));
        self.send(requests).await?;
        Ok(meal_plan_event_from_pb(&event))
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
        let data = self.get_user_data().await?;
        let stored = stored_event(&data, calendar_id, event_id)?;
        let event = PbCalendarEvent {
            date: Some(date.to_string()),
            title: title.map(str::to_string),
            recipe_id: recipe_id.map(str::to_string),
            label_id: label_id.map(str::to_string),
            ..stored.clone()
        };
        let mut ids = self.ids();
        let mut requests =
            operations::update_event(&mut ids, stored, event.clone(), calendar_events(&data));
        let recipe = event.recipe_id.as_deref().and_then(|id| {
            data.recipe_data_response
                .as_ref()?
                .recipes
                .iter()
                .find(|r| r.identifier == id)
        });
        if let (Some(recipe), Some((list_id, items))) = (recipe, recipe_ingredients_list(&data)) {
            requests.extend(operations::update_event_ingredients_on_list(
                &mut ids,
                data.list_context(list_id),
                items,
                recipe,
                stored,
                &event,
            ));
        }
        self.send(requests).await
    }

    /// Delete a meal plan event
    ///
    /// # Arguments
    ///
    /// * `calendar_id` - The ID of the meal planning calendar
    /// * `event_id` - The ID of the event to delete
    pub async fn delete_meal_plan_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let stored = stored_event(&data, calendar_id, event_id)?;
        let requests =
            operations::delete_event(&mut self.ids(), stored, recipe_ingredients_list(&data));
        self.send(requests).await
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
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| AnyListError::Other(format!("Invalid start date: {}", e)))?;
        let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| AnyListError::Other(format!("Invalid end date: {}", e)))?;
        let data = self.get_user_data().await?;
        let items = data
            .shopping_list(list_id)
            .map(|l| l.items.as_slice())
            .ok_or_else(|| AnyListError::NotFound(format!("List with ID {} not found", list_id)))?;

        let mut events: Vec<&PbCalendarEvent> = calendar_events(&data)
            .iter()
            .filter(|e| {
                e.date
                    .as_deref()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .is_some_and(|d| d >= start && d <= end)
            })
            .collect();
        events.sort_by_key(|e| (e.date.clone(), e.order_added_sort_index.unwrap_or_default()));

        let recipes = data
            .recipe_data_response
            .as_ref()
            .map(|r| r.recipes.as_slice())
            .unwrap_or_default();
        let ingredients: Vec<_> = events
            .iter()
            .filter_map(|event| {
                let recipe_id = event.recipe_id.as_deref()?;
                let recipe = recipes.iter().find(|r| r.identifier == recipe_id)?;
                Some(
                    recipe
                        .ingredients
                        .iter()
                        .filter(|i| i.is_heading != Some(true))
                        .map(move |i| operations::event_item_ingredient(recipe, i, event)),
                )
            })
            .flatten()
            .collect();

        let requests = operations::add_ingredients_to_list(
            &mut self.ids(),
            data.list_context(list_id),
            items,
            data.starter_list(list_id, pb_starter_list::Type::FavoriteItems),
            data.starter_list(list_id, pb_starter_list::Type::RecentItems),
            &ingredients,
        );
        self.send(requests).await
    }
}

fn calendar_events(data: &PbUserDataResponse) -> &[PbCalendarEvent] {
    data.meal_planning_calendar_response
        .as_ref()
        .map(|c| c.events.as_slice())
        .unwrap_or_default()
}

fn stored_event<'a>(
    data: &'a PbUserDataResponse,
    calendar_id: &str,
    event_id: &str,
) -> Result<&'a PbCalendarEvent> {
    calendar_events(data)
        .iter()
        .find(|e| e.identifier == event_id)
        .ok_or_else(|| {
            AnyListError::NotFound(format!(
                "Event {} not found in calendar {}",
                event_id, calendar_id
            ))
        })
}

/// The list recipe ingredients were last added to in the apps, and its
/// items.
fn recipe_ingredients_list(data: &PbUserDataResponse) -> Option<(&str, &[PbListItem])> {
    let list_id = data
        .mobile_app_settings_response
        .as_ref()?
        .list_id_for_recipe_ingredients
        .as_deref()
        .filter(|id| !id.is_empty())?;
    data.shopping_list(list_id)
        .map(|l| (list_id, l.items.as_slice()))
}

fn meal_plan_event_from_pb(event: &PbCalendarEvent) -> MealPlanEvent {
    MealPlanEvent {
        id: event.identifier.clone(),
        calendar_id: event.calendar_id.clone(),
        date: event.date.clone().unwrap_or_default(),
        title: event.title.clone(),
        recipe_id: event.recipe_id.clone(),
        label_id: event.label_id.clone(),
        details: event.details.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meal_plan_event_from_pb_maps_all_fields() {
        let pb = PbCalendarEvent {
            identifier: "event-1".to_string(),
            logical_timestamp: Some(1),
            calendar_id: Some("calendar-1".to_string()),
            date: Some("2026-06-15".to_string()),
            title: Some("Dinner".to_string()),
            details: Some("notes".to_string()),
            recipe_id: Some("recipe-1".to_string()),
            label_id: Some("label-1".to_string()),
            order_added_sort_index: Some(0),
            recipe_scale_factor: Some(1.0),
            ..Default::default()
        };

        let event = meal_plan_event_from_pb(&pb);

        assert_eq!(event.id(), "event-1");
        assert_eq!(event.calendar_id(), Some("calendar-1"));
        assert_eq!(event.date(), "2026-06-15");
        assert_eq!(event.title(), Some("Dinner"));
        assert_eq!(event.recipe_id(), Some("recipe-1"));
        assert_eq!(event.label_id(), Some("label-1"));
        assert_eq!(event.details(), Some("notes"));
    }

    #[test]
    fn test_meal_plan_event_from_pb_handles_missing_optionals() {
        let pb = PbCalendarEvent {
            identifier: "event-2".to_string(),
            ..Default::default()
        };

        let event = meal_plan_event_from_pb(&pb);

        assert_eq!(event.id(), "event-2");
        assert_eq!(event.calendar_id(), None);
        assert_eq!(event.date(), "");
        assert_eq!(event.title(), None);
        assert_eq!(event.recipe_id(), None);
        assert_eq!(event.label_id(), None);
        assert_eq!(event.details(), None);
    }
}
