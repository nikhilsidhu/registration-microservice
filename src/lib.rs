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
