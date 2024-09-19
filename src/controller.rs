use actix_web::web;
use actix_ws::Session;
use futures_util::lock::Mutex;
use serde::Deserialize;

use crate::{
    entity::Book,
    repository::{BooksRepository, Repository},
};

#[derive(Deserialize)]
#[serde(tag = "action")]
pub enum ActionTypes {
    #[serde(rename(deserialize = "update_book"))]
    UpdateBook { id: String, book: Book },
    #[serde(rename(deserialize = "add_book"))]
    AddBook { book: Book },
    #[serde(rename(deserialize = "get_book"))]
    GetBook { id: String },
    #[serde(rename(deserialize = "delete_book"))]
    DeleteBook { id: String },
    #[serde(rename(deserialize = "get_books"))]
    GetBooks,
}

pub async fn do_action(
    repository: web::Data<Mutex<BooksRepository>>,
    action: ActionTypes,
    mut session: Session,
) -> () {
    match action {
        ActionTypes::AddBook { book } => {
            let book = repository.lock().await.add(book);

            session
                .text(serde_json::to_string(&book).unwrap())
                .await
                .unwrap();

            ()
        }
        ActionTypes::UpdateBook { id, book } => {
            match repository.lock().await.update(id, book) {
                Ok(book) => {
                    session
                        .text(serde_json::to_string(&book).unwrap())
                        .await
                        .unwrap();
                }
                Err(err) => {
                    session
                        .text(serde_json::to_string(&err).unwrap())
                        .await
                        .unwrap();
                }
            };

            ()
        }
        ActionTypes::GetBook { id } => {
            match repository.lock().await.find_by_id(id) {
                Ok(book) => {
                    session
                        .text(serde_json::to_string(&book).unwrap())
                        .await
                        .unwrap();
                }
                Err(err) => {
                    session
                        .text(serde_json::to_string(&err).unwrap())
                        .await
                        .unwrap();
                }
            };

            ()
        }
        ActionTypes::DeleteBook { id } => {
            match repository.lock().await.delete(id) {
                Ok(book) => {
                    session
                        .text(serde_json::to_string(&book).unwrap())
                        .await
                        .unwrap();
                }
                Err(err) => {
                    session
                        .text(serde_json::to_string(&err).unwrap())
                        .await
                        .unwrap();
                }
            };

            ()
        }
        ActionTypes::GetBooks => {
            let books = repository.lock().await.find_all();

            session
                .text(serde_json::to_string(&books).unwrap())
                .await
                .unwrap();

            ()
        }
    }
}
