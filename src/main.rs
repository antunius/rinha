mod pessoa;
pub(crate) mod entity;

use std::any::Any;
use std::env;
use std::fmt::Display;
use std::ops::{Add, Deref};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest, ResponseError, post};
use actix_web::error::HttpError;
use actix_web::web::{Data, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, Iden, NotSet, PaginatorTrait, QueryFilter, Statement, Unset};
use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use crate::entity::pessoa::{ActiveModel, Model};
use crate::pessoa::Pessoa;
use crate::entity::pessoa::Entity as PessoaEntity;
use std::string::String;
use std::time::Duration;
use actix_web::dev::ResourcePath;
use uuid::Uuid;

#[derive(Deserialize)]
struct QueryTerm {
    t: String,
}

#[get("/{user_id}")]
async fn get_by_id(path: web::Path<(String)>, db: Data<AppState>) -> impl Responder {
    let id = match Uuid::try_parse(path.into_inner().as_str()) {
        Ok(uuid) => { uuid }
        Err(_) => { return HttpResponse::BadRequest().finish(); }
    };
    let pessoa = get_pessoa(db, id).await;
    match pessoa {
        None => { HttpResponse::NotFound().finish() }
        Some(p) => { HttpResponse::Ok().json(<Model as Into<Pessoa>>::into(p)) }
    }
}

async fn get_pessoa(db: Data<AppState>, id: Uuid) -> Option<Model> {
    let pessoa = PessoaEntity::find_by_id(id)
        .one(&db.conn).await.unwrap();
    pessoa
}

#[get("")]
async fn get_by_terms(t: web::Query<(QueryTerm)>, db: Data<AppState>) -> Result<impl Responder, HttpError> {
    let mut term = format!("%{}%", t.t);
    let pessoa = PessoaEntity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DatabaseBackend::Postgres, r#"SELECT * FROM pessoa
WHERE search LIKE $1 limit 50"#, [term.into()]))
        .all(&db.conn)
        .await
        .unwrap();

    let x: Vec<Pessoa> = pessoa
        .iter()
        .map(|model| (*model).clone().into())
        .collect();

    Ok(Json(x))
}

#[get("/contagem-pessoas")]
async fn contagem(db: Data<AppState>) -> Result<impl Responder, HttpError> {
    let count = PessoaEntity::find()
        .count(&db.conn).await.unwrap();

    Ok(Json(count))
}

#[post("")]
async fn create(pessoa: Json<Pessoa>, data: Data<AppState>) -> impl Responder {
    match validate(pessoa.0) {
        Ok(p) => {
            let x = save_pessoa(&data, p).await;
            match x {
                Ok(_entity) => { HttpResponse::Created().insert_header(("LOCATION", format!("/pessoas/{}", _entity.id.unwrap().to_string()))).finish() }
                Err(_error) => { HttpResponse::UnprocessableEntity().finish() }
            }
        }
        Err(_) => { HttpResponse::UnprocessableEntity().finish() }
    }
}

async fn save_pessoa(data: &Data<AppState>, p: Pessoa) -> Result<ActiveModel, DbErr> {
    let x = ActiveModel {
        id: NotSet,
        apelido: Set(p.apelido.to_owned().unwrap()),
        nome: Set(p.nome.clone().unwrap()),
        nascimento: Set(p.nascimento.to_owned()),
        stack: Set( get_stack(p)),
        search: NotSet,
    }.save(&data.conn).await;
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
    let db = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let max_conn = env::var("MAX_CONN").expect("MAX_CONN is not set in .env file");
    let conn_timeout = env::var("TIMEOUT_CONN").expect("MAX_CONN is not set in .env file");
    let conn_timeout_acquire = env::var("ACQUIRE_CONN").expect("MAX_CONN is not set in .env file");
    let conn_timeout_idle = env::var("IDLE_CONN").expect("MAX_CONN is not set in .env file");
    let conn_max_lifetime = env::var("LIFETIME_CONN").expect("MAX_CONN is not set in .env file");

    /*
        let mut opt = ConnectOptions::new(&db);
        opt.max_connections(max_conn.parse().unwrap())
            .min_connections(max_conn.parse().unwrap())
            .connect_timeout(Duration::from_secs(conn_timeout.parse().unwrap()))
            .acquire_timeout(Duration::from_secs(conn_timeout_acquire.parse().unwrap()))
            .idle_timeout(Duration::from_secs(conn_timeout_idle.parse().unwrap()))
            .max_lifetime(Duration::from_secs(conn_max_lifetime.parse().unwrap()))
            .set_schema_search_path("public"); // Setting default PostgreSQL schema



     */
    let db = Database::connect(db).await.expect("Error to connect db");
    let server = HttpServer::new(move || {
        App::new().app_data(Data::new(AppState { conn: db.clone() }))
            .default_service(web::to(|| HttpResponse::NotFound()))
            .service(web::scope("/pessoas")
                .service(get_by_id)
                .service(create)
                .service(get_by_terms))
            .service(contagem)
    }).bind((format!("{}:{}", host, port).path()))?;

    server.run().await?;
    Ok(())
}

#[derive(Debug, Clone)]
struct AppState {
    pub(crate) conn: DatabaseConnection,
}

