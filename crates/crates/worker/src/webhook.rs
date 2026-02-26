//! This file is reponsible to call registered webhooks when an event is received from the message queue.

use std::collections::HashMap;

use crate::InputValue;

/// Represents a registered webhook and the blueprint for the outgoing request.
#[derive(Clone, Debug)]
pub struct Webhook {
    /// The destination endpoint for the webhook.
    pub url: String,

    /// The HTTP method (GET, POST, etc.) to be used.
    pub request_type: reqwest::Method,

    /// Key-value pairs for HTTP headers.
    /// Using a HashMap ensures unique header names and fast lookups.
    pub headers: HashMap<String, InputValue>,

    /// URL query parameters (e.g., ?key=value).
    pub parameters: HashMap<String, InputValue>,

    /// Data to be sent in the JSON body of the request.
    pub json_body: HashMap<String, InputValue>,
}

impl Webhook {
    pub fn build(self, client: &reqwest::Client) -> Result<reqwest::Request, reqwest::Error> {
        // Process query parameters
        let mut url = reqwest::Url::parse(&self.url).expect("Invalid URL");
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in &self.parameters {
                let param_value = value.to_string();
                query_pairs.append_pair(key, &param_value);
            }
        }
        let mut request = client.request(self.request_type.clone(), url);

        // Process headers
        for (key, value) in &self.headers {
            let header_value = value.to_string();
            request = request.header(key, header_value);
        }
        // Process JSON body
        if !self.json_body.is_empty() {
            let mut json_body = serde_json::Map::new();
            for (key, value) in &self.json_body {
                let body_value = value.to_json_value();
                json_body.insert(key.clone(), body_value);
            }

            request = request.json(&json_body);
        }

        request.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_build() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            InputValue::Constant("Bearer token".into()),
        );

        let mut parameters = HashMap::new();
        parameters.insert(
            "query".to_string(),
            InputValue::Constant("search term".into()),
        );

        let mut json_body: HashMap<String, InputValue> = HashMap::new();
        json_body.insert("key".to_string(), InputValue::Constant("value".into()));

        let webhook = Webhook {
            url: "https://webhook.site/14fe219b-e355-489f-87e1-e1b6aec84722".into(),
            request_type: reqwest::Method::GET,
            headers,
            parameters,
            json_body,
        };

        let client = reqwest::Client::new();
        let result = webhook.build(&client);
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(
            request.url().as_str(),
            "https://webhook.site/14fe219b-e355-489f-87e1-e1b6aec84722?query=search+term"
        );
        assert_eq!(request.method(), reqwest::Method::GET);
        assert_eq!(
            request.headers().get("Authorization").unwrap(),
            "Bearer token"
        );
        assert_eq!(
            request.body().unwrap().as_bytes(),
            Some(r#"{"key":"value"}"#.as_bytes())
        );

        let response = reqwest::Client::new()
            .execute(request.try_clone().unwrap())
            .await
            .unwrap();

        assert!(response.status().is_success());
    }
}
