use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    time::SystemTime,
};

use crate::{
    entity::Book,
    error::{code::NOT_FOUND, SocketError},
};

pub(crate) trait Repository<Entity> {
    fn new() -> Self;
    fn add(&mut self, entity: Entity) -> Entity;
    fn find_all(&self) -> Vec<Entity>;
    fn find_by_id(&self, id: String) -> Result<Entity, SocketError>;
    fn update(&mut self, id: String, entity: Entity) -> Result<Entity, SocketError>;
    fn delete(&mut self, id: String) -> Result<Entity, SocketError>;
}

pub(crate) struct BooksRepository {
    books: HashMap<String, Book>,
}

impl Repository<Book> for BooksRepository {
    fn new() -> Self {
        BooksRepository {
            books: HashMap::new(),
        }
    }

    fn add(&mut self, entity: Book) -> Book {
        let mut hasher = DefaultHasher::new();

        let timestamp = SystemTime::now();

        timestamp.hash(&mut hasher);

        let id = hasher.finish().to_string();

        let entity = Book {
            id: Some(id.clone()),
            ..entity
        };

        self.books.insert(id.clone(), entity.clone());

        entity
    }

    fn find_all(&self) -> Vec<Book> {
        self.books
            .iter()
            .map(|id_and_book| id_and_book.1.clone())
            .collect()
    }

    fn find_by_id(&self, id: String) -> Result<Book, SocketError> {
        match self.books.get(&id) {
            Some(book) => Ok(book.clone()),
            None => Err(SocketError {
                code: NOT_FOUND,
                message: format!("Book with id: {} not found", id.clone()),
            }),
        }
    }

    fn update(&mut self, id: String, entity: Book) -> Result<Book, SocketError> {
        let book = match self.books.get_mut(&id) {
            Some(book) => book,
            None => {
                return Err(SocketError {
                    code: NOT_FOUND,
                    message: format!("Book with id: {} not found", id.clone()),
                })
            }
        };

        book.title = entity.title;
        book.author = entity.author;
        book.year = entity.year;

        Ok(book.clone())
    }

    fn delete(&mut self, id: String) -> Result<Book, SocketError> {
        match self.books.remove(&id) {
            Some(book) => Ok(book),
            None => Err(SocketError {
                code: NOT_FOUND,
                message: format!("Book with id: {} not found", id.clone()),
            }),
        }
    }
}
