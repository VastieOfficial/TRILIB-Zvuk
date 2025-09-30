use std::panic::AssertUnwindSafe;
use std::{ env, error::Error, path::PathBuf, time::Duration};

use axum::routing::post;
use axum::Json;
use axum::{response::IntoResponse, Router};
use axum::extract::DefaultBodyLimit;
use hyper::StatusCode;
use once_cell::sync::Lazy;
use reqwest::{Client};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{fs::{self}, time::timeout};


async fn get_url(id: &str, auth_cookie: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::new();

    let uri = format!(
        "https://zvuk.com/api/v1/graphql"
    );

    let body = json!({
        "query": "query getStream($ids: [ID!]!, $quality: String, $encodeType: String, $includeFlacDrm: Boolean!) {
        mediaContents(ids: $ids, quality: $quality, encodeType: $encodeType) {
            ... on Track {
            stream {
                expire
                high
                mid
                flacdrm @include(if: $includeFlacDrm)
            }
            }
            ... on Episode {
            stream {
                expire
                mid
            }
            }
            ... on Chapter {
            stream {
                expire
                mid
            }
            }
        }
        }",
        "operationName": "getStream",
        "variables": {
            "quality": "hq",
            "encodeType": "wv",
            "includeFlacDrm": false,
            "ids": [id],
        }
    });
    let res = client
        .post(&uri)
        .body(body.to_string())
        .header("Cookie", auth_cookie)
        .header("content-type", "application/json")
        .header("Accept", "application/graphql-response+json, application/json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Spotify API error: {}", res.status()).into());
    }
    let x: String = res.text().await?;
    
    let json: Value = serde_json::from_str(&x)?;

    let stream = &json["data"]["mediaContents"][0]["stream"];
    
    let url_high: Option<&str> = stream["high"].as_str();
    let url_mid = stream["mid"].as_str();
    Ok(vec![url_high.unwrap().to_string(), url_mid.unwrap().to_string()])
}

async fn dl_file(url: &str, to: &str) {
    let resp = reqwest::get(url).await.expect("request failed");
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned);
    let bytes = resp.bytes().await.expect("failed to read body");

    let ext = ct
        .and_then(|ct| ct.parse::<mime::Mime>().ok())
        .and_then(|mime| mime_guess::get_mime_extensions(&mime))
        .and_then(|guess| guess.first().cloned())
        .unwrap_or_default();

    let final_path = if ext.is_empty() {
        to.to_string()
    } else {
        format!("{}.{}", to, ext)
    };

    tokio::fs::write(final_path, bytes)
        .await
        .expect("failed to write file");
}

static CACHEDIR: Lazy<PathBuf> = Lazy::new(|| {
    env::var("TRI_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = env::current_dir().unwrap();
            path.push("TRICACHE");
            path
        })
});

static PORT: Lazy<u16> = Lazy::new(|| {
    env::var("TRI_ZVUK_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3501)
});

async fn save_by_id(id: &str, auth_cookie: &str, hash: &str)  -> Result<bool, Box<dyn Error>> {
    let urls = get_url(id, auth_cookie).await.expect("couldn't get stream");

    for (i, format) in ["best", "mid"].iter().enumerate() {
        let mut filepath = (*CACHEDIR).clone();
        filepath.push(hash);
        filepath.push("zvuk");
        tokio::fs::create_dir_all(&filepath).await.unwrap();
        filepath.push(format);

        if let Some(url) = urls.get(i) {
            dl_file(url, filepath.to_str().unwrap()).await;
        }
    }
    return Ok(true);
}


async fn download(
    Json(payload): Json<DownloadZVUK>,
) -> impl IntoResponse {
    let result = timeout(Duration::from_secs(300), async move {
        let run = AssertUnwindSafe(async move {
            save_by_id(&payload.id, &payload.auth_cookie, &payload.hash)
                .await
                .map_err(|e| anyhow!("save_best_medium_low failed: {}", e))?;

            Ok::<(), anyhow::Error>(())
        })
        .await;

        let _res: Result<(), anyhow::Error> = match run {
            Ok(inner) => Ok(inner),
            Err(panic) => Err(anyhow!("panic: {:?}", panic)),
        };
    })
    .await;

    match result {
        Ok(_inner) => (
            StatusCode::OK,
            axum::Json(IsOK { ok: true, error: "".to_string() }),
        ),
        Err(_panic) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(IsOK { ok: false, error: _panic.to_string() }),
        ),
    }
}

#[derive(Deserialize)]
struct DownloadZVUK {
    id: String,
    hash: String,
    auth_cookie: String,
}

#[derive(Serialize)]
struct IsOK {
    ok: bool,
    error: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/dl", post(download))
        .layer(DefaultBodyLimit::max(1024 * 1024));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", *PORT))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}