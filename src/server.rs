use actix::{Addr, System};
use actix_web::dev::Payload;
use actix_web::error::MultipartError;
use actix_web::http::{self, header, StatusCode};
use actix_web::multipart::MultipartItem;
use actix_web::{
  middleware, server, App, Error as WebError, HttpMessage, HttpRequest, HttpResponse,
};
use askama::Template;
use chrono::{DateTime, Utc};
use failure::Error;
use futures::{future, Future, Stream};
use indexmap::IndexMap;
use log::debug;
use registration_microservice::queue_actor::{QueueActor, QueueHandler, SendMessage, TaskId};
use registration_microservice::{Request, Response, REQUESTS, RESPONSES};
use std::fmt;
use std::sync::{Arc, Mutex};

// This will hold our tasks and their status
type SharedTasks = Arc<Mutex<IndexMap<String, Record>>>;

/*
* [STRUCT] Record
* holds info about our tasks
* task_id -> unique id
* timestamp -> when task was posted
* status -> task status
*/
#[derive(Clone)]
struct Record {
  task_id: TaskId,
  timestamp: DateTime<Utc>,
  status: Status,
}

/*
* [ENUM] Status
* two variants for each task
*   -> InProgress or Done
*   -> Done means worker returned a Response
*/
#[derive(Clone)]
enum Status {
  InProgress,
  Done(Response),
}

/*
* We can use a display trait for status to update our HTML template
*/
impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Status::InProgress => write!(f, "in progress"),
      Status::Done(resp) => match resp {
        Response::Succeed(data) => write!(f, "done: {}", data),
        Response::Failed(err) => write!(f, "failed: {}", err),
      },
    }
  }
}

/*
* [STRUCT] State
* the server has shared state
* tasks -> the shared list of tasks
* addr -> address of Actor/Handler
*/
#[derive(Clone)]
struct State {
  tasks: SharedTasks,
  addr: Addr<QueueActor<ServerHandler>>,
}

/*
* [STRUCT] ServerHandler
* keeps a copy of the shared tasks
*/
struct ServerHandler {
  tasks: SharedTasks,
}

/*
* [TRAIT] QueueHandler
* incoming() -> responses from the workers
* outgoing() -> requests for the worker
* handle() -> updates the status of tasks
*/
impl QueueHandler for ServerHandler {
  type Incoming = Response;
  type Outgoing = Request;

  fn incoming(&self) -> &str {
    RESPONSES
  }
  fn outgoing(&self) -> &str {
    REQUESTS
  }
  fn handle(
    &self,
    id: &TaskId,
    incoming: Self::Incoming,
  ) -> Result<Option<Self::Outgoing>, Error> {
    debug!("Result returned: {:?}", incoming);
    self.tasks.lock().unwrap().get_mut(id).map(move |rec| {
        rec.status = Status::Done(incoming);
    });
    Ok(None)
  }
}

/*
* [FUNCITON] index_handler()
* returns Ok http response with name of microservice
*/
fn index_handler(_: &HttpRequest<State>) -> HttpResponse {
  HttpResponse::Ok().body("Camping Registration Microservice")
}

/*
* [FUNCTION] tasks_handler()
* renders requests part of tasks struct
*/
fn tasks_handler(req: HttpRequest<State>) -> impl Future<Item = HttpResponse, Error = WebError> {
  let tasks: Vec<_> = req
    .state()
    .tasks
    .lock()
    .unwrap()
    .values()
    .cloned()
    .collect();
  let tmpl = Tasks { tasks };
  future::ok(HttpResponse::Ok().body(tmpl.render().unwrap()))
}

/*
* [STRUCT] Tasks
* struct for our requests
*/
#[derive(Template)]
#[template(path = "register.html")]
struct Tasks {
  tasks: Vec<Record>,
}

fn main() {
  
}