use super::{ensure_queue, spawn_client};
use actix::fut::wrap_future;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler, SystemRunner};
use failure::{format_err, Error};
use futures::Future;
use lapin::channel::{BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel};
use lapin::error::Error as LapinError;
use lapin::message::Delivery;
use lapin::types::{FieldTable, ShortString};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use uuid::Uuid;

pub type TaskId = ShortString;