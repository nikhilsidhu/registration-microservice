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

/*
  [TRAIT] QueueHandler
  * static lifetime b/c instances will be used
    as fields of actors (also have static lifetime)
  * Incoming -> incoming message type (must be deserializable)
  * Outgoing -> outgoing message type (must be serializable)
  * incoming() -> gets name of queue to comsume incoming messages
  * outgoing() -> gets name of queue actor will send messages to
  * handle() -> returns the result with optional Outgoing instance
                  if None is returned then no messages will be sent
*/
pub trait QueueHandler: 'static {
  type Incoming: for<'de> Deserialize<'de>;
  type Outgoing: Serialize;

  fn incoming(&self) -> &str;
  fn outgoing(&self) -> &str;
  fn handle(
    &self,
    id: &TaskId,
    incoming: Self::Incoming,
  ) -> Result<Option<Self::Outgoing>, Error>;
}

/*
  [STRUCT] QueueActor
  * the connection to RabbitMQ is build over a TcpStream
  * handler must implement QueueHandler trait
*/
pub struct QueueActor<T: QueueHandler> {
  channel: Channel<TcpStream>,
  handler: T,
}