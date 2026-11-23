pub struct PlateMasker;

impl PlateMasker {
    /// Mascara a placa para logs conforme LGPD (`"ABC1234"` -> `"ABC****"`).
    pub fn mask(plate: &str) -> String {
        let normalized = plate.trim().to_uppercase();
        let char_count = normalized.chars().count();
        if char_count < 4 {
            return "****".to_string();
        }
        let prefix: String = normalized.chars().take(char_count - 4).collect();
        format!("{}****", prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_masker_standard_and_mercosul() {
        assert_eq!(PlateMasker::mask("ABC1234"), "ABC****");
        assert_eq!(PlateMasker::mask(" bra2e19 "), "BRA****");
        assert_eq!(PlateMasker::mask("AB1"), "****");
        assert_eq!(PlateMasker::mask(""), "****");
    }
}
