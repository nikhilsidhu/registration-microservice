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

fn main() {
  
}