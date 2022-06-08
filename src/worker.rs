use actix::System;
use failure::{format_err, Error};
use log::debug;
use qr_microservice::queue_actor::{QueueActor, QueueHandler, TaskId};
use qr_microservice::{Request, Response, REQUESTS, RESPONSES};

