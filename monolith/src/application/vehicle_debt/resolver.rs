use crate::domain::debt::contracts::VehicleDebtProvider;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProviderResolver {
    providers: HashMap<String, Arc<dyn VehicleDebtProvider>>,
    default_order: Vec<String>,
}

impl ProviderResolver {
    pub fn new(
        providers: HashMap<String, Arc<dyn VehicleDebtProvider>>,
        default_order: Vec<String>,
    ) -> Self {
        Self {
            providers,
            default_order,
        }
    }

    pub fn resolve(&self, name: &str) -> Option<Arc<dyn VehicleDebtProvider>> {
        let normalized = name.trim().to_lowercase();
        self.providers.get(&normalized).cloned()
    }

    pub fn get_configured_order(&self) -> Vec<(String, Arc<dyn VehicleDebtProvider>)> {
        self.resolve_order(None)
    }

    pub fn resolve_order(
        &self,
        custom_order: Option<&[String]>,
    ) -> Vec<(String, Arc<dyn VehicleDebtProvider>)> {
        let order = custom_order.unwrap_or(&self.default_order);
        let mut resolved = Vec::new();

        for name in order {
            let key = name.trim().to_lowercase();
            if let Some(provider) = self.providers.get(&key) {
                resolved.push((key, Arc::clone(provider)));
            }
        }

        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::debt::entities::ProviderDebtResponse;
    use crate::domain::debt::errors::ProviderError;
    use async_trait::async_trait;

    struct DummyProvider(&'static str);

    #[async_trait]
    impl VehicleDebtProvider for DummyProvider {
        fn name(&self) -> &'static str {
            self.0
        }

        async fn get_debts(
            &self,
            plate: &str,
            _request_id: Option<&str>,
        ) -> Result<ProviderDebtResponse, ProviderError> {
            Ok(ProviderDebtResponse::new(plate, vec![], self.0))
        }
    }

    #[test]
    fn test_resolver_filters_registered_providers_in_order() {
        let mut map: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        map.insert("rest".to_string(), Arc::new(DummyProvider("rest")));
        map.insert("soap".to_string(), Arc::new(DummyProvider("soap")));
        let csv = Arc::new(DummyProvider("ai"));
        map.insert("csv".to_string(), Arc::clone(&csv) as Arc<dyn VehicleDebtProvider>);
        map.insert("ai".to_string(), csv);

        let resolver = ProviderResolver::new(
            map,
            vec![
                "rest".to_string(),
                "unknown".to_string(),
                "soap".to_string(),
                "csv".to_string(),
            ],
        );

        let ordered = resolver.get_configured_order();
        let names: Vec<&str> = ordered.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["rest", "soap", "csv"]);
    }
}
