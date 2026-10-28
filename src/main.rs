use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};

use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};


const PATH:&str = "scores.csv";
const ADDR:&str = "127.0.0.1:3030";
const ALLOWED: &[&str] = &[
    "https://race.iwaservice.uk",
    "https://race.iwa.me.uk",
];
const NAME:[&str;2] = ["easy","hard"];


#[derive(Clone, Deserialize, Serialize)]
struct Score{
    name:String,
    time:f32
}
#[derive(Clone)]
struct Mode{
    easy:Db,
    hard:Db,
}
impl Mode {
    fn get(&self,name:&str) -> Option<&Db>{
        match name{
            "easy" => Some(&self.easy),
            "hard" => Some(&self.hard),
            _ => None,
        }
    }
}
#[derive(Serialize)]
struct Ranking {
    scores:Vec<Score>,
}

type Db = Arc<Mutex<Vec<Score>>>;

#[tokio::main]
async fn main() {
    let db:Mode = Mode {easy:Arc::new(Mutex::new(load(NAME[0]))),hard:Arc::new(Mutex::new(load(NAME[1])))};

    let cors =  CorsLayer::new()
    .allow_origin(AllowOrigin::predicate(|origin, _| {
        ALLOWED.iter().any(|a| origin.as_bytes() == a.as_bytes())
            || origin.as_bytes().starts_with(b"http://localhost:")
    }))
    .allow_methods(Any)
    .allow_headers(Any);

    let app = Router::new()
        .route("/scores/{mode}", get(list).post(add))
        .with_state(db)
        .layer(cors);
    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
fn load(name:&str) -> Vec<Score>{
    match csv::Reader::from_path(format!("{}_{}",name,PATH)) {
        Ok(mut reader) => reader.deserialize().filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}
fn append(score:&Score,name:&str) -> Result<(),Box<dyn  std::error::Error>>{
    let need_header = std::fs::metadata(format!("{}_{}",name,PATH))
        .map(|m| m.len() == 0)
        .unwrap_or(true);
    let file = OpenOptions::new().create(true).append(true).open(format!("{}_{}",name,PATH))?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(need_header)
        .from_writer(file);

    writer.serialize(score)?;
    writer.flush()?;
    Ok(())
}
async fn add(Path(mode):Path<String>,State(state):State<Mode>,Json(score):Json<Score>)->StatusCode{
    match state.get(&mode){
        Some(modes) => {
            let mut db_arc = modes.lock().unwrap();
            if let Err(e) = append(&score,&mode){
                eprintln!("csv write Err:{}",e);
            }
            db_arc.push(score);
            StatusCode::CREATED
        },
        None => StatusCode::NOT_FOUND,
    }
}
async fn list(Path(mode):Path<String>,State(state): State<Mode>) -> Result<Json<Ranking>,StatusCode>{
    match state.get(&mode){
        Some(modes) => {
            let mut scores = modes.lock().unwrap().clone();
            scores.sort_by(|a,b|a.time.total_cmp(&b.time));
            scores.truncate(10);
            Ok(Json(Ranking { scores }))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}