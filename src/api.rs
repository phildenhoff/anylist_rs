use crate::protobuf::anylist::PbUserDataResponse;
use prost::Message;
use reqwest::header;

pub async fn get_user_data(
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
