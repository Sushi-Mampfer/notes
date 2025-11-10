use reqwest::Client;
use serde::Serialize;
use serde_json::{Value};

#[derive(Serialize)]
struct Query {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    options: Options
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String
}

#[derive(Serialize)]
struct Options {
    temperature: f32,
    top_p: f32,
    repeat_penalty: f32
}

pub async fn sumarize(text: String) -> String {
    let query = Query {
        model: "qwen2.5:14b".to_string(),
        messages: vec![Message { role: "system".to_string(), content: include_str!("summarize_prompt.txt").to_string() }, Message { role: "user".to_string(), content: text }],
        stream: false,
        options: Options { temperature: 0.2, top_p: 0.9, repeat_penalty: 1.1 }
    };
    let res: Value = Client::new().post("http://localhost:11434/api/chat").json(&query).send().await.unwrap().json().await.unwrap();
    res.get("message").unwrap().get("content").unwrap().to_string()
}