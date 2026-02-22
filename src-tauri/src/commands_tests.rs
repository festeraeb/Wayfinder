#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    // Ensure tokio runtime available
    #[tokio::test]
    async fn test_validate_azure_config_mock_success() {
        let deployment = "text-embedding-3-small";
        let api_version = "2023-05-15";
        let path = format!("/openai/deployments/{}/embeddings", deployment);

        let _m = mock("POST", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [{"embedding": [0.1, 0.2]}]}"#)
            .create();

        let endpoint = server_url();
        // call validate_azure_config using the mock server
        let res = validate_azure_config("/tmp/index".to_string(), endpoint.clone(), "fake-key".to_string(), deployment.to_string(), Some(api_version.to_string())).await;
        assert!(res.is_ok());
        let value = res.unwrap();
        assert!(value.get("success").and_then(|v| v.as_bool()).unwrap_or(false));
        // final_url should contain server_url and api_version
        let final_url = value.get("final_url").and_then(|v| v.as_str()).unwrap_or("");
        assert!(final_url.contains(&endpoint));
        assert!(final_url.contains(api_version));
    }
}