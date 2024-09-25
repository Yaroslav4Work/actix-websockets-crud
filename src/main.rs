use actix_web::{web, App, HttpServer};
use controller::route;
use entity::Book;
use futures_util::lock::Mutex;
use repository::{BooksRepository, Repository as _};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;

mod controller;
mod entity;
mod error;
mod repository;

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
                // Unwrap is only for tests, because... behavior is predictable
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

                // Unwrap is only for tests, because... behavior is predictable
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
                // Unwrap is only for tests, because... behavior is predictable
                let deleted_book = repository.lock().await.delete(id).unwrap();

                assert!(deleted_book.title == created_book.title);
                assert!(repository.lock().await.find_all().len() == 0);
            }
            None => panic!("Id in created book not found"),
        };
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let repository = web::Data::new(Mutex::new(BooksRepository::new()));

    let mut hasher = DefaultHasher::new();

    let timestamp = SystemTime::now();

    timestamp.hash(&mut hasher);

    let id = Some(hasher.finish().to_string());

    repository.lock().await.add(Book {
        id,
        title: "Test Book 1".to_string(),
        author: "I am".to_string(),
        year: 2024,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&repository))
            .service(route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
