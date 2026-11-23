use crate::domain::debt::entities::ProviderDebtResponse;
use crate::domain::debt::errors::ProviderError;
use async_trait::async_trait;

#[async_trait]
pub trait VehicleDebtProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn get_debts(
        &self,
        plate: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError>;
}
