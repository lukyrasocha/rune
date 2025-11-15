use reqwest::Client;
use serde_json::{json, Value};
use futures_core::Stream;
use futures_util::TryStreamExt;
use std::pin::Pin;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;  


pub struct Ollama {
    base_url: String,
    model: String,
    client: Client,
}

impl Ollama {
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model, client: Client::new() }
    }
}

pub trait LLM {
    async fn chat_stream(&self, messages: &[Value]) -> Result<Pin<Box<dyn Stream<Item=Result<Value, anyhow::Error>> + Send + '_>>, anyhow::Error>;
}

impl LLM for Ollama {
    async fn chat_stream(&self, messages: &[Value]) -> Result<Pin<Box<dyn Stream<Item=Result<Value, anyhow::Error>> + Send + '_>>, anyhow::Error> {
        let body = json!({"model": self.model, "messages": messages, "stream": true});
        let response = self.client.post(format!("{}/api/chat", self.base_url)).json(&body).send().await?;

        // Convert response body to async reader and frame it by lines
        let byte_stream = response.bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
        let reader = StreamReader::new(byte_stream);
        let lines = FramedRead::new(reader, LinesCodec::new());
        
        // Parse each line as JSON
        let stream = lines.map_err(anyhow::Error::from).and_then(|line: String| async move {
            let value = serde_json::from_str::<Value>(&line)?;
            Ok(value)
        });
        
        Ok(Box::pin(stream))
    }
}