use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::lists::{transform_api_list_item, ListItem};
use crate::operations;
use crate::protobuf::anylist::{pb_starter_list, PbListItem, PbUserDataResponse};

impl AnyListClient {
    /// Add an item to a shopping list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to add the item to
    /// * `name` - The name of the item
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anylist_rs::AnyListClient;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = AnyListClient::login("user@example.com", "password")
    ///     .await
    ///     .expect("Failed to authenticate");
    ///
    /// client.add_item("list-id", "Milk")
    ///     .await
    ///     .expect("Failed to add item");
    /// # }
    /// ```
    pub async fn add_item(&self, list_id: &str, name: &str) -> Result<ListItem> {
        self.add_item_with_details(list_id, name, None, None, None)
            .await
    }

    /// Add an item to a shopping list with additional details
    ///
    /// The item is categorised like the web client does it: the list's
    /// remembered category for an item of that name wins, then `category`,
    /// then the list's default category.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to add the item to
    /// * `name` - The name of the item
    /// * `quantity` - Optional quantity (e.g., "2", "1 lb", "500 g")
    /// * `details` - Optional additional details/notes
    /// * `category` - Optional category: a system category such as "dairy",
    ///   or the name of one of the list's categories
    pub async fn add_item_with_details(
        &self,
        list_id: &str,
        name: &str,
        quantity: Option<&str>,
        details: Option<&str>,
        category: Option<&str>,
    ) -> Result<ListItem> {
        let data = self.get_user_data().await?;
        let list = data.list_context(list_id);
        let mut ids = self.ids();
        let mut item = operations::new_list_item(&mut ids, list, name, category);
        item.details = details.map(str::to_string);
        item.quantity_pb = quantity.map(operations::parse_quantity);

        self.send(operations::add_item(&mut ids, list, item.clone()))
            .await?;
        Ok(list_item(item))
    }

    /// Update an existing item
    ///
    /// Only what changed is sent, one operation per field, as the web client
    /// does. `None` leaves a field as it is.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to update
    /// * `name` - The new name for the item
    /// * `quantity` - Optional new quantity
    /// * `details` - Optional new details
    /// * `category` - Optional new category: a system category such as
    ///   "dairy", or the name of one of the list's categories
    pub async fn update_item(
        &self,
        list_id: &str,
        item_id: &str,
        name: &str,
        quantity: Option<&str>,
        details: Option<&str>,
        category: Option<&str>,
    ) -> Result<()> {
        let data = self.get_user_data().await?;
        let mut item = stored_item(&data, list_id, item_id)?.clone();
        let list = data.list_context(list_id);
        let mut ids = self.ids();
        let mut requests = Vec::new();

        if item.name.as_deref() != Some(name) {
            requests.extend(operations::set_item_name(&mut ids, &item, name));
            item.name = Some(name.to_string());
        }
        if let Some(details) = details {
            if item.details.as_deref().unwrap_or_default() != details {
                requests.extend(operations::set_item_details(&mut ids, &item, details));
                item.details = Some(details.to_string());
            }
        }
        if let Some(quantity) = quantity {
            let quantity = operations::parse_quantity(quantity);
            if item.quantity_pb.clone().unwrap_or_default() != quantity {
                requests.extend(operations::set_item_quantity(
                    &mut ids,
                    &item,
                    quantity.clone(),
                ));
                item.quantity_pb = Some(quantity);
            }
        }
        if let Some(category) = category {
            let category = list.find_category(category).ok_or_else(|| {
                AnyListError::NotFound(format!(
                    "Category '{}' not found in list {}",
                    category, list_id
                ))
            })?;
            let current = item
                .category_assignments
                .iter()
                .find(|a| a.category_group_id == category.category_group_id)
                .and_then(|a| a.category_id.as_ref());
            if current != category.identifier.as_ref() {
                requests.extend(operations::set_item_category(
                    &mut ids, list, &item, category,
                ));
            }
        }

        self.send(requests).await
    }

    /// Delete an item from a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to delete
    pub async fn delete_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        self.bulk_delete_items(list_id, &[item_id]).await
    }

    /// Delete multiple items from a list in a single operation
    ///
    /// Like the web client, the deleted items are added to the list's Recent
    /// Items.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the items
    /// * `item_ids` - The IDs of the items to delete
    pub async fn bulk_delete_items(&self, list_id: &str, item_ids: &[&str]) -> Result<()> {
        if item_ids.is_empty() {
            return Ok(());
        }

        let data = self.get_user_data().await?;
        let items: Vec<PbListItem> = list_items(&data, list_id)?
            .iter()
            .filter(|item| item_ids.contains(&item.identifier.as_str()))
            .cloned()
            .collect();
        if items.is_empty() {
            return Err(AnyListError::NotFound(
                "No matching items found".to_string(),
            ));
        }

        let recent_items = data.starter_list(list_id, pb_starter_list::Type::RecentItems);
        let requests = operations::delete_items(&mut self.ids(), list_id, &items, recent_items);
        self.send(requests).await
    }

    /// Cross off (check) an item on a list
    ///
    /// Like the web client, this also adds the item to the list's Recent
    /// Items.
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to cross off
    pub async fn cross_off_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        self.set_item_checked(list_id, item_id, true).await
    }

    /// Uncheck (uncross) an item on a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list containing the item
    /// * `item_id` - The ID of the item to uncheck
    pub async fn uncheck_item(&self, list_id: &str, item_id: &str) -> Result<()> {
        self.set_item_checked(list_id, item_id, false).await
    }

    /// Set the checked status of an item
    async fn set_item_checked(&self, list_id: &str, item_id: &str, checked: bool) -> Result<()> {
        let data = self.get_user_data().await?;
        let item = stored_item(&data, list_id, item_id)?;
        let recent_items = data.starter_list(list_id, pb_starter_list::Type::RecentItems);
        let requests = operations::set_item_checked(&mut self.ids(), item, checked, recent_items);
        self.send(requests).await
    }

    /// Delete all crossed-off (checked) items from a list
    ///
    /// # Arguments
    ///
    /// * `list_id` - The ID of the list to clear crossed-off items from
    pub async fn delete_all_crossed_off_items(&self, list_id: &str) -> Result<()> {
        let data = self.get_user_data().await?;
        let items: Vec<PbListItem> = list_items(&data, list_id)?
            .iter()
            .filter(|item| item.checked == Some(true))
            .cloned()
            .collect();
        if items.is_empty() {
            return Ok(());
        }

        let recent_items = data.starter_list(list_id, pb_starter_list::Type::RecentItems);
        let requests =
            operations::delete_crossed_off_items(&mut self.ids(), list_id, &items, recent_items);
        self.send(requests).await
    }
}

fn list_items<'a>(data: &'a PbUserDataResponse, list_id: &str) -> Result<&'a [PbListItem]> {
    data.shopping_list(list_id)
        .map(|list| list.items.as_slice())
        .ok_or_else(|| AnyListError::NotFound(format!("List with ID {} not found", list_id)))
}

fn stored_item<'a>(
    data: &'a PbUserDataResponse,
    list_id: &str,
    item_id: &str,
) -> Result<&'a PbListItem> {
    data.list_item(list_id, item_id).ok_or_else(|| {
        AnyListError::NotFound(format!("Item {} not found in list {}", item_id, list_id))
    })
}

fn list_item(item: PbListItem) -> ListItem {
    transform_api_list_item(vec![item])
        .pop()
        .expect("a new item has a name and list")
}
