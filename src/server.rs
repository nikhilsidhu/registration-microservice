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

fn main() {
  
}