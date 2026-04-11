use axum::{routing::post, Json, Router};
use reqwest::{Client, multipart};
use serde::{Deserialize};
use std::{collections::HashSet, env};
use tokio::time::{sleep, Duration};
use thiserror::Error;

const API_URL: &str = "https://7tv.io/v3/gql";

#[derive(Error, Debug)]
pub enum SevenTvError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Missing field in response")]
    MissingField,

    #[error("Env error: {0}")]
    MissingEnv(#[from] std::env::VarError),
}

#[derive(Deserialize, Debug)]
struct Emote {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct EmoteSetResponse {
    data: EmoteSetData,
}

#[derive(Deserialize)]
struct EmoteSetData {
    emote_set: EmoteSet,
}

#[derive(Deserialize, Debug)]
struct EmoteSet {
    id: String,
    name: String,
    emotes: Vec<Emote>,
}

async fn gql(
    client: &Client,
    token: &str,
    query: &str,
    variables: serde_json::Value,
) -> Result<serde_json::Value, SevenTvError> {
    let res = client
        .post(API_URL)
        .bearer_auth(token)
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;

    Ok(res.json().await?)
}

async fn get_emote_set(
    client: &Client,
    token: &str,
    set_id: &str,
) -> Result<EmoteSet, SevenTvError> {
    let query = r#"
        query($id: ObjectID!) {
            emote_set(id: $id) {
                id
                name
                emotes {
                    id
                    name
                }
            }
        }
    "#;

    let json = gql(
        client, 
        token, 
        query, 
        serde_json::json!({ "id": set_id }),
    )
    .await?;

    let parsed: EmoteSetResponse =
        serde_json::from_value(json).map_err(|_| SevenTvError::InvalidResponse)?;

    Ok(parsed.data.emote_set)
}

async fn create_set(
    client: &Client,
    token: &str,
    name: &str,
) -> Result<String, SevenTvError> {
    let query = r#"
        mutation($name: String!) {
            create_emote_set(name: $name) {
                id
            }
        }
    "#;

    let json = gql(
        client,
        token,
        query,
        serde_json::json!({ "name": name }),
    )
    .await?;

    let id = match json["data"]["create_emote_set"]["id"].as_str() {
        Some(id) => id.to_string(),
        None => return Err(SevenTvError::MissingField),
    };
    
    Ok(id)
}

async fn add_emote(
    client: &Client,
    token: &str,
    set_id: &str,
    emote_id: &str,
    name: &str,
) -> Result<(), SevenTvError> {
    let query = r#"
        mutation($set_id: ObjectID!, $emote_id: ObjectID!, $name: String!) {
            emote_set(id: $set_id) {
                add_emote(id: $emote_id, name: $name) {
                    id
                }
            }
        }
    "#;

    gql(
        client,
        token,
        query,
        serde_json::json!({
            "set_id": set_id,
            "emote_id": emote_id,
            "name": name
        }),
    )
    .await?;

    Ok(())
}

async fn reupload_emote(
    client: &Client,
    token: &str,
    set_id: &str,
    emote: &Emote,
) -> Result<(), SevenTvError> {
    let urls = [
        format!("https://cdn.7tv.app/emote/{}/4x.gif", emote.id),
        format!("https://cdn.7tv.app/emote/{}/4x.webp", emote.id),
    ];

    let mut bytes = None;

    for url in urls {
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(b) = res.bytes().await {
                bytes = Some(b);
                break;
            }
        }
    }

    let bytes = match bytes {
        Some(b) => b,
        None => return Err(SevenTvError::InvalidResponse),
    };

    let form = multipart::Form::new()
        .text("name", emote.name.clone())
        .part("file", multipart::Part::bytes(bytes.to_vec()).file_name("emote"));

    let res = client
        .post("https://7tv.io/v3/emotes")
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;

    let new_id = match json["id"].as_str() {
        Some(id) => id,
        None => return Err(SevenTvError::MissingField),
    };

    add_emote(client, token, set_id, new_id, &emote.name).await?;

    Ok(())
}

async fn clone_set(
    client: &Client,
    token: &str,
    source_set_id: &str,
    new_name: &str,
) -> Result<String, SevenTvError> {
    let source = get_emote_set(client, token, source_set_id).await?;
    let new_set_id = create_set(client, token, new_name).await?;

    let mut seen = HashSet::new();

    for emote in source.emotes {
        if !seen.insert(emote.name.clone()) {
            continue;
        }

        println!("copying emote: {}", emote.name);

        if let Err(_) = add_emote(client, token, &new_set_id, &emote.id, &emote.name).await {
            eprintln!("fallback upload for {}", emote.name);
            reupload_emote(client, token, &new_set_id, &emote).await?;
        }

        sleep(Duration::from_millis(150)).await;
    }

    Ok(new_set_id)
}

// added mod emote_copy to main.rs 

#[derive(Deserialize)]
struct CopyRequest {
    source_set_id: String,
    new_name: String,
}

async fn copy_handler(Json(payload): Json<CopyRequest>) -> String {
    let client = Client::new();

    let token = match env::var("SEVENTV_TOKEN") {
        Ok(t) => t,
        Err(_) => return "Missing SEVENTV_TOKEN".into(),
    };

    match clone_set(
        &client,
        &token,
        &payload.source_set_id,
        &payload.new_name,
    )
    .await
    {
        Ok(id) => format!("Created new set: {}", id),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn seventv_router() -> Router {
    Router::new().route("/copy-set", post(copy_handler))
}
// router added to main.rs