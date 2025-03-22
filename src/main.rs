use actix_web::{web, App, HttpServer, Responder, HttpResponse, post, get};
use serde::Deserialize;
use std::env;
use std::process::Command;
use std::path::Path;
use dotenv::dotenv;

#[derive(Deserialize)]
struct DeployRequest {
    secret: String,
}

async fn deploy_service(service_name: &str) -> Result<String, String> {
    let commands_dir = env::var("COMMANDS_DIR").unwrap_or_else(|_| "./command".to_string());
    let script_path = format!("{}/{}.sh", commands_dir, service_name);

    if !Path::new(&script_path).exists() {
        return Err("Script not found".to_string());
    }

    let status = Command::new(script_path)
        .status()
        .map_err(|_| "Deployment failed".to_string())?;

    if status.success() {
        Ok("Deployment successful".to_string())
    } else {
        Err("Deployment failed".to_string())
    }
}

#[post("/deploy/{service_name}")]
async fn deploy(service_name: web::Path<String>, req: web::Json<DeployRequest>) -> impl Responder {
    let webhook_secret = env::var("WEBHOOK_SECRET").expect("WEBHOOK_SECRET must be set in .env!");

    if req.secret != webhook_secret {
        return HttpResponse::Forbidden().json(serde_json::json!({ "error": "Unauthorized" }));
    }

    match deploy_service(&service_name).await {
        Ok(message) => HttpResponse::Ok().json(serde_json::json!({ "status": message })),
        Err(err) if err == "Script not found" => HttpResponse::NotFound().json(serde_json::json!({ "error": err })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": err })),
    }
}

#[get("/")]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .service(deploy)
            .service(hello_world)
    })
    .bind("0.0.0.0:3794")?
    .run()
    .await
}
