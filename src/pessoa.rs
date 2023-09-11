use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use std::option::Option;
use crate::entity::pessoa::Model;

#[derive(Deserialize, Serialize)]
pub struct Pessoa {
    pub apelido: Option<String>,
    pub nome: Option<String>,
    pub nascimento: NaiveDate,
    pub stack: Option<Vec<String>>,
}

impl From<Model> for Pessoa {
    fn from(model: Model) -> Self {
        Pessoa {
            apelido: Some(model.apelido),
            nome: Some(model.nome),
            nascimento: model.nascimento.into(),
            stack: Some(model.stack.unwrap_or_default().split_ascii_whitespace().map(String::from).collect()),
        }
    }
}