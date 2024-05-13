use mongodb::bson::oid::ObjectId;

#[derive(Debug, serde::Deserialize)]
pub struct Price {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub template_index: u32,
    pub label: String,
    pub provider: GatewayProvider,
    pub provider_id: String,
    pub live: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayProvider {
    Paypal,
    Stripe,
}

impl From<GatewayProvider> for shared::database::GatewayProvider {
    fn from(value: GatewayProvider) -> Self {
        match value {
            GatewayProvider::Paypal => Self::Paypal,
            GatewayProvider::Stripe => Self::Stripe,
        }
    }
}
