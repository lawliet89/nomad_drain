use std::error::Error;

use aws_lambda_events::event::autoscaling::AutoScalingEvent as Event;
use lambda_runtime::{error::HandlerError, lambda, Context};
use serde::Serialize;

#[derive(Serialize)]
struct HandlerResult {
    pub message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(lambda_handler);

    Ok(())
}

fn lambda_handler(_event: Event, _context: Context) -> Result<HandlerResult, HandlerError> {
    Ok(HandlerResult {
        message: "Hello world".to_string(),
    })
}

// Environment deserialize https://github.com/softprops/envy
