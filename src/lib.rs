pub mod queue_actor;
 
use actix::{Message, SystemRunner};
use failure::Error;
use futures::Future;
use lapin::channel::{Channel, QueueDeclareOptions};
use lapin::client::{Client, ConnectionOptions};
use lapin::error::Error as LapinError;
use lapin::queue::Queue;
use lapin::types::FieldTable;
use serde_derive::{Deserialize, Serialize};
use tokio::net::TcpStream;

pub const REQUESTS: &str = "requests";
pub const RESPONSES: &str = "responses";

/*
* [FUNCTION] spawn_client()
* creates a Client and creates a Channel from it
*   -> using a TcpStream (constant address)
*   -> the Client is connected to RabbitMQ
* we execute the connect future immediately using block_on()
*   -> returns the Client and a Heartbeat instance
*   -> the heartbeat pings RabbitMQ as part of the event loop
*
* [PARAM] sys (&mut SystemRunner) -> runs our systems event loop
* [RETURN] channel -> the created Channel instance
*/
pub fn spawn_client(sys: &mut SystemRunner) -> Result<Channel<TcpStream>, Error> {
  // TODO: make spawn_client() take an address param
  let addr = "127.0.0.1:5672".parse().unwrap();
  let fut = TcpStream::connect(&addr)
  .map_err(Error::from)
  .and_then(|stream| {
    let options = ConnectionOptions::default();
    Client::connect(stream, options).from_err::<Error>()
  });
  let (client, heartbeat) = sys.block_on(fut)?;
  actix::spawn(heartbeat.map_err(drop));
  let channel = sys.block_on(client.create_channel())?;
  Ok(channel)
}