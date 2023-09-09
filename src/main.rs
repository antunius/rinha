mod pessoa;
pub(crate) mod entity;

use std::any::Any;
use std::env;
use std::fmt::Display;
use std::time::Duration;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, ResponseError, post};
use actix_web::error::HttpError;
use actix_web::web::{Data, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, Iden, NotSet, QueryFilter, Statement};
use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use crate::entity::pessoa::{ActiveModel, Model};
use crate::pessoa::Pessoa;
use crate::entity::pessoa::Entity as PessoaEntity;
use std::string::String;
use uuid::Uuid;

#[derive(Deserialize)]
struct QueryTerm {
    t: String,
}

#[get("/{user_id}")]
async fn get_by_id(path: web::Path<(String)>, db: web::Data<AppState>) -> impl Responder {
    let id = Uuid::try_parse(path.into_inner().as_str()).expect("");
    let pessoa = get_pessoa(&db, id).await;
    match pessoa {
        None => { HttpResponse::NotFound().finish() }
        Some(p) => { HttpResponse::Ok().json(<Model as Into<Pessoa>>::into(p)) }
    }
}

async fn get_pessoa(db: &Data<AppState>, id: Uuid) -> Option<Model> {
    let pessoa = PessoaEntity::find_by_id(id)
        .one(&db.conn.to_owned()).await
        .expect("");
    pessoa
}

#[get("/")]
async fn get_by_terms(t: web::Query<(QueryTerm)>, db: Data<AppState>) -> Result<impl Responder, HttpError> {
    let mut term = format!("%{}%", t.t);
    let pessoa = PessoaEntity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DatabaseBackend::Postgres, r#"SELECT * FROM pessoa
WHERE apelido LIKE $1
   or nome LIKE $1
   or stack like $1"#, [term.into()]))
        .all(&db.conn.to_owned())
        .await
        .expect("Error to Get");

    let x: Vec<Pessoa> = pessoa
        .iter()
        .map(|model| (*model).clone().into())
        .collect();

    Ok(Json(x))
}

#[post("/")]
async fn create(pessoa: Json<Pessoa>, data: Data<AppState>) -> impl Responder {
    match validate(pessoa.0) {
        Ok(p) => {
            let x = save_pessoa(&data, p).await;
            match x {
                Ok(_entity) => { HttpResponse::Created() }
                Err(_error) => { HttpResponse::UnprocessableEntity() }
            }
        }
        Err(_) => { HttpResponse::UnprocessableEntity() }
    }
}

async fn save_pessoa(data: &Data<AppState>, p: Pessoa) -> Result<ActiveModel, DbErr> {
    let x = ActiveModel {
        id: NotSet,
        apelido: Set(p.apelido.to_owned().unwrap()),
        nome: Set(p.nome.clone().unwrap()),
        nascimento: Set(p.nascimento.to_owned()),
        stack: Set(get_stack(p)),
    }.save(&data.conn.to_owned()).await;
    x
}

fn validate(pessoa: Pessoa) -> Result<Pessoa, String, > {
    if pessoa.nome.is_none() || pessoa.apelido.is_none() {
        Err("Campos invalidos".parse().unwrap())
    } else { Ok(pessoa) }
}

fn get_stack(pessoa: Pessoa) -> Option<String> {
    match pessoa.stack.clone() {
        None => { None }
        Some(s) => { Some(s.join(",")) }
    }
}

async fn not_found(request: HttpRequest) -> impl Responder {
    HttpResponse::NotFound().body(format!("Not Found {}", request.path()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let db = env::var("DATABASE_URL").expect("HOST is not set in .env file");

    let mut opt = ConnectOptions::new(&db);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .set_schema_search_path("public"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await.expect("Error to connect database");
    let server = HttpServer::new(move || {
        App::new().app_data(web::Data::new(AppState { conn: db.clone() }))
            .default_service(web::to(|| HttpResponse::NotFound()))
            .service(web::scope("/pessoas")
                .service(get_by_id)
                .service(create)
                .service(get_by_terms))
    }).workers(10).bind(("127.0.0.1", 8080))?;

    server.run().await?;
    Ok(())
}

#[derive(Debug, Clone)]
struct AppState {
    pub(crate) conn: DatabaseConnection,
}

