#[cfg(test)]
mod tests {
    use crate::analyze_file_ftaql;
    use serde_json::Value;

    #[test]
    fn test_analyze_project() {
        let source_code = "let a = 1;";
        let result = analyze_file_ftaql(source_code, false, false);
        let value: Result<Value, _> = serde_json::from_str(&result);
        assert!(value.is_ok());
    }
}
