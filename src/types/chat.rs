use std::collections::HashMap;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct ChatHistory {
    
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn from_json(json: &Value) -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Chat {
    
}

impl Chat {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn from_json(json: &Value) -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    pub text: String,
    pub is_final: bool,
    pub safety_truncated: bool,
    pub create_time: Option<String>
}

impl Candidate {
    pub fn new(id: impl Into<String>, text: impl Into<String>, is_final: bool, safety_truncated: bool, create_time: Option<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            is_final,
            safety_truncated,
            create_time
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let blank = json!("");
        let blank_bool = json!(false);

        Self::new(
            json.get("candidate_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("raw_content").unwrap_or(&blank).as_str().unwrap(),
            json.get("is_final").unwrap_or(&blank_bool).as_bool().unwrap_or(false),
            json.get("safety_truncated").unwrap_or(&blank_bool).as_bool().unwrap_or(false),
            json.get("create_time").and_then(|v| v.as_str().map(String::from))
        )
    }
}

#[derive(Debug, Clone)]
pub struct Turn {
    pub id: String,
    pub chat_id: String,
    pub create_time: Option<String>,
    pub last_update_time: Option<String>,
    pub state: Value,
    pub author_id: i64,
    pub author_name: String,
    pub author_is_human: bool,
    pub primary_candidate_id: Option<String>,
    pub candidates: HashMap<String, Candidate>
}

impl Turn {
    pub fn new(id: impl Into<String>, chat_id: impl Into<String>, create_time: Option<String>, last_update_time: Option<String>, state: Value, author_id: i64, author_name: impl Into<String>, author_is_human: bool, primary_candidate_id: Option<String>, candidates: HashMap<String, Candidate>) -> Self {
        Self {
            id: id.into(),
            chat_id: chat_id.into(),
            create_time,
            last_update_time,
            state,
            author_id,
            author_name: author_name.into(),
            author_is_human,
            primary_candidate_id,
            candidates
        }
    }

    pub fn from_json(json: &Value) -> Self {
        let t = json.get("turn_key").expect("turn_key key should be present within JSON Value provided");
        let author = t.get("author").expect("author key should be present within turn_key object in the JSON Value provided");

        let blank = json!("");

        Self::new(
            t.get("candidate_id").unwrap_or(&blank).as_str().unwrap(),
            t.get("chat_id").unwrap_or(&blank).as_str().unwrap(),
            json.get("create_time").and_then(|v| v.as_str().map(String::from)),
            json.get("create_time").and_then(|v| v.as_str().map(String::from)),
            json.get("state").unwrap_or(&blank).clone(),
            author.get("author_id").unwrap_or(&json!(0)).as_i64().unwrap_or(0),
            author.get("name").unwrap_or(&blank).as_str().unwrap_or(""),
            author.get("is_human").unwrap_or(&json!(false)).as_bool().unwrap_or(false),
            json.get("primary_candidate_id").and_then(|v| v.as_str().map(String::from)),
            json.get("candidates").unwrap_or(&json!([])).as_array().unwrap_or(&vec![]).into_iter().map(|v| Candidate::from_json(&v)).map(|v| (v.id.clone(), v)).collect()
        )
    }

    pub fn get_candidates(&self) -> Vec<Candidate> {
        self.candidates.values().cloned().collect()
    } 

    pub fn get_primary_candidate(&self) -> Option<&Candidate> {
        if let Some(id) = &self.primary_candidate_id {
            self.candidates.get(id)
        } else {
            None
        }
    }
}