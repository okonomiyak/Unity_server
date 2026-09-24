use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};

const PATH:&str = "scores.csv";

#[derive(Clone, Deserialize, Serialize)]
struct Score{
    name:String,
    time:f32
}
#[derive(Serialize)]
struct Ranking {
    scores:Vec<Score>,
}

type Db = Arc<Mutex<Vec<Score>>>;

#[tokio::main]
async fn main() {
    let db:Db = Arc::new(Mutex::new(load()));

    let app = Router::new()
        .route("/scores", get(list).post(add))
        .with_state(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
fn load() -> Vec<Score>{
    match csv::Reader::from_path(PATH) {
        Ok(mut reader) => reader.deserialize().filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}
fn append(score:&Score) -> Result<(),Box<dyn  std::error::Error>>{
    let need_header = std::fs::metadata(PATH)
        .map(|m| m.len() == 0)
        .unwrap_or(true);
    let file = OpenOptions::new().create(true).append(true).open(PATH)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(need_header)
        .from_writer(file);

    writer.serialize(score)?;
    writer.flush()?;
    Ok(())
}
async fn add(State(db):State<Db>,Json(score):Json<Score>){
    let mut db = db.lock().unwrap();
    if let Err(e) = append(&score){
        eprintln!("csv write Err:{}",e);
    }
    db.push(score);
}
async fn list(State(db): State<Db>) -> Json<Ranking>{
    let mut scores = db.lock().unwrap().clone();
    scores.sort_by(|a,b|a.time.total_cmp(&b.time));
    scores.truncate(10);
    Json(Ranking { scores })
}