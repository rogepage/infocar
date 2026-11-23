use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Registro de métricas operacionais em memória utilizando contadores atômicos lock-free (`AtomicU64`).
#[derive(Debug, Default)]
pub struct SimpleMetricsRegistry {
    vehicle_debt_requests_total: AtomicU64,
    vehicle_debt_success_total: AtomicU64,
    vehicle_debt_error_total: AtomicU64,
    provider_requests_total: AtomicU64,
    provider_failures_total: AtomicU64,
    provider_retries_total: AtomicU64,
    provider_fallbacks_total: AtomicU64,
}

impl SimpleMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self, metric: &str) {
        if let Some(counter) = self.counter_for(metric) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get(&self, metric: &str) -> u64 {
        self.counter_for(metric)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn get_all(&self) -> BTreeMap<String, u64> {
        let mut map = BTreeMap::new();
        map.insert(
            "vehicle_debt_requests_total".to_string(),
            self.vehicle_debt_requests_total.load(Ordering::Relaxed),
        );
        map.insert(
            "vehicle_debt_success_total".to_string(),
            self.vehicle_debt_success_total.load(Ordering::Relaxed),
        );
        map.insert(
            "vehicle_debt_error_total".to_string(),
            self.vehicle_debt_error_total.load(Ordering::Relaxed),
        );
        map.insert(
            "provider_requests_total".to_string(),
            self.provider_requests_total.load(Ordering::Relaxed),
        );
        map.insert(
            "provider_failures_total".to_string(),
            self.provider_failures_total.load(Ordering::Relaxed),
        );
        map.insert(
            "provider_retries_total".to_string(),
            self.provider_retries_total.load(Ordering::Relaxed),
        );
        map.insert(
            "provider_fallbacks_total".to_string(),
            self.provider_fallbacks_total.load(Ordering::Relaxed),
        );
        map
    }

    pub fn reset(&self) {
        self.vehicle_debt_requests_total.store(0, Ordering::Relaxed);
        self.vehicle_debt_success_total.store(0, Ordering::Relaxed);
        self.vehicle_debt_error_total.store(0, Ordering::Relaxed);
        self.provider_requests_total.store(0, Ordering::Relaxed);
        self.provider_failures_total.store(0, Ordering::Relaxed);
        self.provider_retries_total.store(0, Ordering::Relaxed);
        self.provider_fallbacks_total.store(0, Ordering::Relaxed);
    }

    fn counter_for(&self, metric: &str) -> Option<&AtomicU64> {
        match metric {
            "vehicle_debt_requests_total" => Some(&self.vehicle_debt_requests_total),
            "vehicle_debt_success_total" => Some(&self.vehicle_debt_success_total),
            "vehicle_debt_error_total" => Some(&self.vehicle_debt_error_total),
            "provider_requests_total" => Some(&self.provider_requests_total),
            "provider_failures_total" => Some(&self.provider_failures_total),
            "provider_retries_total" => Some(&self.provider_retries_total),
            "provider_fallbacks_total" => Some(&self.provider_fallbacks_total),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry_increment_get_and_reset() {
        let registry = SimpleMetricsRegistry::new();
        registry.increment("provider_requests_total");
        registry.increment("provider_requests_total");
        registry.increment("provider_retries_total");
        registry.increment("unknown_metric");

        assert_eq!(registry.get("provider_requests_total"), 2);
        assert_eq!(registry.get("provider_retries_total"), 1);
        assert_eq!(registry.get("provider_fallbacks_total"), 0);

        let all = registry.get_all();
        assert_eq!(all.len(), 7);
        assert_eq!(all["provider_requests_total"], 2);

        registry.reset();
        assert_eq!(registry.get("provider_requests_total"), 0);
    }
}
