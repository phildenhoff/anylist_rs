use crate::protobuf::anylist::PbUserDataResponse;
use prost::Message;
use reqwest::header;

async fn get_user_data(
    signed_user_id: &str,
) -> Result<PbUserDataResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://www.anylist.com/data/user-data/get")
        .header(header::CONTENT_TYPE, "application/x-protobuf")
        .header("X-AnyLeaf-Signed-User-ID", signed_user_id)
        .send()
        .await?;

    let bytes = res.bytes().await?;
    let data = PbUserDataResponse::decode(bytes.as_ref())?;
    Ok(data)
}

pub async fn get_lists(
    signed_user_id: &str,
) -> Result<PbUserDataResponse, Box<dyn std::error::Error>> {
    let data = get_user_data(signed_user_id).await?;
    match data.shopping_lists_response {
        Some(ref res) => {
            let list_names: Vec<String> = res
                .new_lists
                .iter()
                .map(|list| list.name.clone())
                .filter(|name| name.is_some())
                .map(|name| name.unwrap())
                .collect::<Vec<_>>();
            println!("{:?}", list_names)
        }
        None => println!("No shopping lists found"),
    }

    Ok(data)
}
