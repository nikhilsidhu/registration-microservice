use actix::System;
use failure::{format_err, Error};
use log::debug;
use qr_microservice::queue_actor::{QueueActor, QueueHandler, TaskId};
use qr_microservice::{Request, Response, REQUESTS, RESPONSES};

// Worker has no state
struct WorkerHandler {}

/*
* [TRAIT] QueueHandler
* 
*/
impl QueueHandler for WokerHandler {
  type Incoming = QrRequest;
  type Outgoing = QrResponse;

  fn incoming(&self) -> &str {
    REQUESTS
  }
  fn outgoing(&self) -> &str {
    RESPONSES
  }
  fn handle(
    &self,
    _: &TaskId,
    incoming: Self::Incoming,
  ) -> Result<Option<Self::Outgoing>, Error> {
    debug!("In: {:?}", incoming);
    // TODO: call helper function (not yet implemented)
    
    debug!("Out: {:?}", outgoing);
    Ok(Some(outgoing))
  }
}

impl WorkerHandler {
  // TODO: implement function called by handler
}

fn main() -> Result<(), Error> {
  env_logger::init(); // for logging
  let mut sys = System::new("registration-worker"); // starts a System instance
  let _ = QueueActor::new(WorkerHandler {}, &mut sys)?; // Creates a QueueActor instance
  let _ = sys.run(); // starts the system
  Ok(())
}