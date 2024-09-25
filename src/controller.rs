use actix_web::{get, rt, web, Error, HttpRequest, HttpResponse};
use actix_ws::{AggregatedMessage, Session};
use futures_util::{lock::Mutex, StreamExt as _};
use log::{debug, error, log_enabled, Level};
use serde::{Deserialize, Serialize};

use crate::{
    entity::Book,
    error::{
        code::{BAD_REQUEST, INTERAL_SERVER_ERROR},
        SocketError,
    },
    repository::{BooksRepository, Repository},
};

#[derive(Deserialize)]
#[serde(tag = "action")]
pub(crate) enum ActionTypes {
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

fn serialize(obj: &impl Serialize) -> String {
    match serde_json::to_string(obj) {
        Ok(json) => json,
        Err(e) => {
            error!("Serialization error {}", e);

            if log_enabled!(Level::Debug) {
                debug!("{}", e);
            }

            let internal_server_error = SocketError {
                code: INTERAL_SERVER_ERROR,
                message: "Internal server error".to_string(),
            };

            match serde_json::to_string(&internal_server_error) {
                Ok(json) => json,
                Err(e) => {
                    error!("Serialization error {} for {:?}", e, internal_server_error);

                    if log_enabled!(Level::Debug) {
                        debug!("{}", e);
                    }

                    format!(
                        "{{code: {}, message: {}}}",
                        INTERAL_SERVER_ERROR, "Internal server error"
                    )
                }
            }
        }
    }
}

async fn send_reponse(session: &mut Session, obj: &impl Serialize) -> () {
    match session.text(serialize(&obj)).await {
        Ok(_) => (),
        Err(e) => error!("Client's connection has been closed: {}", e),
    }
}

async fn do_action(
    repository: web::Data<Mutex<BooksRepository>>,
    action: ActionTypes,
    session: &mut Session,
) -> () {
    let mut repository = repository.lock().await;

    let get_books_action_msg = "get_books_action".to_string();
    let get_books_action_code = 1;

    let get_books_action = SocketError {
        code: get_books_action_code,
        message: get_books_action_msg.clone(),
    };

    let result: Result<Book, SocketError> = match action {
        ActionTypes::AddBook { book } => Ok(repository.add(book)),
        ActionTypes::UpdateBook { id, book } => repository.update(id, book),
        ActionTypes::GetBook { id } => repository.find_by_id(id),
        ActionTypes::DeleteBook { id } => repository.delete(id),
        ActionTypes::GetBooks => Err(get_books_action),
    };

    match result {
        Ok(book) => send_reponse(session, &book).await,
        Err(e) => {
            if e.code == get_books_action_code && e.message == get_books_action_msg {
                return send_reponse(session, &repository.find_all()).await;
            }

            send_reponse(session, &e).await;
        }
    }
}

#[get("/api/v1/")]
pub(crate) async fn route(
    repository: web::Data<Mutex<BooksRepository>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = match actix_ws::handle(&req, stream) {
        Ok(args) => args,
        Err(e) => {
            let err_msg = "Error was occured while try to handle request";

            error!("{}", err_msg);

            if log_enabled!(Level::Debug) {
                debug!("{}", e);
            }

            panic!("{}", err_msg);
        }
    };

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
                            do_action(web::Data::clone(&repository), action, &mut session).await
                        }
                        Err(_) => {
                            let err = SocketError {
                                code: BAD_REQUEST,
                                message: "Incorrect action".to_string(),
                            };

                            send_reponse(&mut session, &err).await;
                        }
                    };
                }

                _ => {
                    let err = SocketError {
                        code: BAD_REQUEST,
                        message: "Bad message type".to_string(),
                    };

                    send_reponse(&mut session, &err).await;
                }
            };
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}
