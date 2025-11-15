mod chat;

use anyhow::*;
use serde_json::json;
use futures_util::StreamExt;
use chat::llm::{Ollama, LLM}; 



#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let ollama_base_url = "http://127.0.0.1:11434";
    let ollama_model = "qwen3:8b";
    let ollama = Ollama::new(ollama_base_url.to_string(), ollama_model.to_string());

    let stdin = std::io::stdin();
    let mut buf = String::new();

    loop {
        buf.clear();
        eprint!("\n> ");
        std::io::Write::flush(&mut std::io::stderr())?;
        if stdin.read_line(&mut buf)? == 0 { break; }
        let user = buf.trim();
        if user.eq_ignore_ascii_case("exit") { break; }

        let messages = vec![
            json!({"role":"system","content":"You are a helpful assistant."}),
            json!({"role":"user","content": user})
        ];

        let mut stream = ollama.chat_stream(&messages).await?;
        eprint!("\n");
        while let Some(ev) = stream.next().await {
            let ev = ev?;
            
            // Ollama returns {"message": {"role": "assistant", "content": "..."}, "done": false}
            if let Some(content) = ev.get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str()) 
            {
                print!("{content}");
                std::io::Write::flush(&mut std::io::stdout())?;
            }
        }
        println!();
    }
    Ok(())
}
