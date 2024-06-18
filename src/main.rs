use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

type UserDb = Arc<Mutex<HashMap<u32, User>>>;

#[get("/greet/{id}")]
async fn greet(userid: web::Path<u32>, db: web::Data<UserDb>) -> impl Responder {
    let db = db.lock().unwrap();
    if let Some(user) = db.get(&userid) {
        format!("Hello, {}", user.name)
    } else {
        format!("User with id {} not found", userid)
    }
}

#[derive(Serialize)]
struct CreateUserResponse {
    id: u32,
    name: String,
}

#[post("/users")]
async fn create_user(
    user_data: web::Json<User>,
    db: web::Data<UserDb>,
) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;
    let name = user_data.name.clone();
    db.insert(new_id, user_data.into_inner());
    HttpResponse::Created().json(CreateUserResponse {
        id: new_id,
        name,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on port {}", port);

    let user_db = Arc::new(Mutex::new(HashMap::<u32, User>::new()));

    HttpServer::new(move || {
        let app_data = web::Data::new(user_db.clone());
        App::new()
            .app_data(app_data)
            .service(greet)
            .service(create_user)
    })
        .bind(("127.0.0.1", port))?
        .workers(2)
        .run()
        .await
}
