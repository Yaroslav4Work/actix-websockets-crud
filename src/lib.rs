use actix_web::{get, http::Error, rt, web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use controller::{do_action, ActionTypes};
use error::{code::BAD_REQUEST, SocketError};
use futures_util::{lock::Mutex, StreamExt as _};
use repository::BooksRepository;

pub mod controller;
pub mod entity;
pub mod error;
pub mod repository;

#[get("/api/v1/")]
pub async fn echo(
    repository: web::Data<Mutex<BooksRepository>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream).unwrap();

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Close(_)) => (),

                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    match serde_json::from_str::<ActionTypes>(&text) {
                        Ok(action) => {
                            do_action(web::Data::clone(&repository), action, session.clone()).await
                        }
                        Err(_) => {
                            let err = SocketError {
                                code: BAD_REQUEST,
                                message: "Incorrect action".to_string(),
                            };

                            session
                                .text(serde_json::to_string(&err).unwrap())
                                .await
                                .unwrap();
                        }
                    };
                }

                _ => {
                    let err = SocketError {
                        code: BAD_REQUEST,
                        message: "Bad message type".to_string(),
                    };

                    session
                        .text(serde_json::to_string(&err).unwrap())
                        .await
                        .unwrap();
                }
            };
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

#[cfg(test)]
mod tests {
    use actix_web::web;
    use futures_util::lock::Mutex;

    use crate::{
        entity::Book,
        repository::{BooksRepository, Repository as _},
    };

    #[actix_web::test]
    async fn test_repository_add_book() {
        let repository = web::Data::new(Mutex::new(BooksRepository::new()));

        let created_book = repository.lock().await.add(Book {
            id: None,
            title: "Test Book 1".to_string(),
            author: "I am".to_string(),
            year: 2024,
        });

        assert!(repository.lock().await.find_all().len() == 1);

        match created_book.id {
            Some(id) => {
                assert!(repository.lock().await.find_by_id(id).unwrap().title == created_book.title)
            }
            None => panic!("Id in created book not found"),
        };
    }

    #[actix_web::test]
    async fn test_repository_update_book() {
        let repository = web::Data::new(Mutex::new(BooksRepository::new()));

        let created_book = repository.lock().await.add(Book {
            id: None,
            title: "Test Book 1".to_string(),
            author: "I am".to_string(),
            year: 2024,
        });

        match created_book.id {
            Some(id) => {
                let title = "Test Book 2".to_string();

                let updated_book = repository
                    .lock()
                    .await
                    .update(
                        id,
                        Book {
                            id: None,
                            title: title.clone(),
                            ..created_book
                        },
                    )
                    .unwrap();

                assert!(updated_book.title == title);
                assert!(updated_book.title != created_book.title);
                assert!(repository.lock().await.find_all().len() == 1);
            }
            None => panic!("Id in created book not found"),
        };
    }

    #[actix_web::test]
    async fn test_repository_delete_book() {
        let repository = web::Data::new(Mutex::new(BooksRepository::new()));

        let title = "Test Book 1".to_string();

        let created_book = repository.lock().await.add(Book {
            id: None,
            title: title.clone(),
            author: "I am".to_string(),
            year: 2024,
        });

        match created_book.id {
            Some(id) => {
                let deleted_book = repository.lock().await.delete(id).unwrap();

                assert!(deleted_book.title == created_book.title);
                assert!(repository.lock().await.find_all().len() == 0);
            }
            None => panic!("Id in created book not found"),
        };
    }
}
