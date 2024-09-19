use actix_web::{web, App, HttpServer};
use actix_websockets_crud::echo;
use actix_websockets_crud::entity::Book;
use actix_websockets_crud::repository::{BooksRepository, Repository};
use futures_util::lock::Mutex;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
