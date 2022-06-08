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
  * Incoming    -> incoming message type (must be deserializable)
  * Outgoing    -> outgoing message type (must be serializable)
  * incoming()  -> gets name of queue to comsume incoming messages
  * outgoing()  -> gets name of queue actor will send messages to
  * handle()    -> [RETURN] the result with optional Outgoing instance
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

/*
  [TRAIT] Actor
  * to become an actor we must implement the actix::Actor trait
  * started() -> for creating the queues
*/
impl<T: QueueHandler> Actor for QueueActor<T> {
  type Context = Context<Self>;

  fn started(&mut self, _: &mut Self::Context) {}
}

/*
  [METHOD] new()
  * we call spawn_client() which will create a Client (connected to the message broker)
  * we then create the two queues we will need (incoming & outgoing)
  * the basic_consume method starts listening for new messages
  *   -> returns a future that is resolved into a stream value
  * we use block_on again to execute this future, resolve it into a stream
  *   and then attach it to the QueueActor we create`
  * 
  * [PARAM] handler (QueueHandler)
  *           -> used to create the queues & used to create QueueActor
  * [PARAM] sys (SystemRunner)
  *           -> blocks to execute future objects immediately
  *           -> which lets us get Result and interrupts other activities if it fails
  * [RETURN] Channel instance
*/
impl<T: QueueHandler> QueueActor<T> {
  pub fn new(handler:T, mut sys: &mut SystemRunner) -> Result<Addr<Self>, Error> {
    let channel = spawn_client(&mut sys)?;
    let chan = channel.clone();
    let fut = ensure_queue(&chan, handler.outgoing());
    sys.block_on(fut)?;
    let fut = ensure_queue(&chan, handler.incoming()).and_then(move |queue| {
      let opts = BasicConsumeOptions {
        ..Defaul::default()
      };
      let table = FieldTable::new();
      let name = format!("{}-consumer", queue.name());
      chan.basic_consume(&queue, &name, opts, table)
    });
    let stream = sys.block_on(fut)?;
    let addr = QueueActor::create(move |ctx| {
      ctx.add_stream(stream);
      Self { channel, handler }
    });
    Ok(addr)
  }
}

/*
  * [TRAIT] StreamHandler
  * the basic_consume method used in QueueHandler returns Delivery
  *   type objects from the queue (Stream)
  * we implement this trait to attach the Stream to QueueActor
  * 
  * [FUNCTION] handle()
  * RabbitMQ expects we acknowledge when we consume a delivered message
  *   -> we use basic_ack() to accomplish this
  *   -> if the process_message doesn't return None, we can use it as a
  *       response message to the outgoing queue (with send_message())
  *
  * [PARAM] item (Delivery)
  *           -> message recieved from the queue
  * [PARAM] ctx (&mut Context<Self>)
  *           -> each actor maintains its internal state through Context
*/
impl<T: QueueHandler> StreamHandler<Delivery, LapinError> for QueueActor<T> {
  fn handle(&mut self, item: Delivery, ctx: &mut Context<Self>) {
    debug!("Message received!");
    let fut = self
      .channel
      .basic_ack(item.delivery_tag, false)
      .map_err(drop);
    ctx.spawn(wrap_future(fut));
    match self.process_message(item, ctx) {
      Ok(pair) => {
        if let Some((corr_id, data)) = pair {
          self.send_message(corr_id, data, ctx);
        }
      }
      Err(err) => {
        warn!("Message processing error: {}", err);
      }
    }
  }
}