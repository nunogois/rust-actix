use actix_cors::Cors;
use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

const REPO: &str = "https://github.com/nunogois/rust-actix";

#[derive(Serialize, Deserialize)]
struct User {
    id: Option<String>,
    name: String,
    email: String,
}

struct AppState {
    users: Mutex<Vec<User>>,
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(format!(
        "Hello world! Check the repo here: <a href=\"{0}\" target=\"_blank\">{0}</a>",
        REPO
    ))
}

#[get("")]
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

#[get("{id}")]
async fn get_user(data: web::Data<AppState>, params: web::Path<String>) -> impl Responder {
    let id = params.into_inner();
    let users = data.users.lock().unwrap();

    if let Some(user) = users.iter().find(|u| u.id == Some(id.to_string())) {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::NotFound().body(format!("ID not found: {}", id))
    }
}

#[post("")]
async fn post_user(data: web::Data<AppState>, mut user: web::Json<User>) -> impl Responder {
    let mut users = data.users.lock().unwrap();

    if let Some(id) = &user.id {
        if users.iter().any(|u| u.id == Some(id.to_string())) {
            return HttpResponse::Conflict().body(format!("ID already exists: {}", id.to_string()));
        }
    } else {
        user.id = Some(Uuid::new_v4().to_string());
    }

    users.push(user.into_inner());

    HttpResponse::Ok().json(users.last())
}

#[put("{id}")]
async fn put_user(
    data: web::Data<AppState>,
    params: web::Path<String>,
    mut user: web::Json<User>,
) -> impl Responder {
    let id = params.into_inner();
    let mut users = data.users.lock().unwrap();

    if !user.id.is_some() {
        user.id = Some(id.to_string());
    }

    if let Some(index) = users.iter().position(|u| u.id == Some(id.to_string())) {
        users[index] = user.into_inner();
        HttpResponse::Ok().json(&users[index])
    } else {
        HttpResponse::NotFound().body(format!("ID not found: {}", id))
    }
}

#[delete("{id}")]
async fn delete_user(data: web::Data<AppState>, params: web::Path<String>) -> impl Responder {
    let id = params.into_inner();
    let mut users = data.users.lock().unwrap();

    if let Some(index) = users.iter().position(|u| u.id == Some(id.to_string())) {
        users.remove(index);
        HttpResponse::Ok().body(format!("ID deleted: {}", id))
    } else if id == "*" {
        if users.len() > 0 {
            users.clear();
            HttpResponse::Ok().body("All users deleted")
        } else {
            HttpResponse::NotFound().body("No users to delete")
        }
    } else {
        HttpResponse::NotFound().body(format!("ID not found: {}", id))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        users: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .service(home)
            .service(
                web::scope("/users")
                    .service(get_users)
                    .service(get_user)
                    .service(post_user)
                    .service(put_user)
                    .service(delete_user),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
