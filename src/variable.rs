use crate::error::{Result, VariableError};
use regex::Regex;
use std::collections::HashMap;

pub fn substitute(text: &str, variables: &HashMap<String, String>) -> Result<String> {
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut result = text.to_string();

    for cap in re.captures_iter(text) {
        let full_match = &cap[0];
        let var_name = &cap[1];
        let value = variables
            .get(var_name)
            .ok_or_else(|| VariableError::UndefinedVariable(var_name.to_string()))?;
        result = result.replace(full_match, value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_single_variable() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "world".to_string());

        let result = substitute("Hello, {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_substitute_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("baseUrl".to_string(), "https://api.example.com".to_string());
        vars.insert("id".to_string(), "42".to_string());

        let result = substitute("{{baseUrl}}/users/{{id}}", &vars).unwrap();
        assert_eq!(result, "https://api.example.com/users/42");
    }

    #[test]
    fn test_substitute_same_variable_multiple_times() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), "1".to_string());

        let result = substitute("{{x}} + {{x}} = 2", &vars).unwrap();
        assert_eq!(result, "1 + 1 = 2");
    }

    #[test]
    fn test_substitute_no_variables() {
        let vars = HashMap::new();
        let result = substitute("No variables here", &vars).unwrap();
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_substitute_undefined_variable() {
        let vars = HashMap::new();
        let result = substitute("Hello, {{name}}!", &vars);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined variable: name"));
    }

    #[test]
    fn test_substitute_in_json() {
        let mut vars = HashMap::new();
        vars.insert("userId".to_string(), "123".to_string());
        vars.insert("title".to_string(), "Test".to_string());

        let json = r#"{"userId": {{userId}}, "title": "{{title}}"}"#;
        let result = substitute(json, &vars).unwrap();
        assert_eq!(result, r#"{"userId": 123, "title": "Test"}"#);
    }

    #[test]
    fn test_substitute_preserves_non_matching_braces() {
        let vars = HashMap::new();
        // Single braces should not be matched
        let result = substitute("{not_a_var}", &vars).unwrap();
        assert_eq!(result, "{not_a_var}");
    }
}
