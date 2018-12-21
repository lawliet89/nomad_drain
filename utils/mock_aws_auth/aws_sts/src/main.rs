//! All this crate does is to return a JSON response to mock AWS STS GetCallerIdentity

extern crate hyper;
extern crate pretty_env_logger;

use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

const XML_RESPONSE: &str = r#"
<GetCallerIdentityResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <GetCallerIdentityResult>
   <Arn>arn:aws:iam::123456789012:user/moto</Arn>
    <UserId>AKIAIOSFODNN7EXAMPLE</UserId>
    <Account>123456789012</Account>
  </GetCallerIdentityResult>
  <ResponseMetadata>
    <RequestId>01234567-89ab-cdef-0123-456789abcdef</RequestId>
  </ResponseMetadata>
</GetCallerIdentityResponse>
"#;

fn main() {
    pretty_env_logger::init();
    let addr = ([0, 0, 0, 0], 8000).into();

    let server = Server::bind(&addr)
        .serve(|| {
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            service_fn_ok(move |_: Request<Body>| Response::new(Body::from(XML_RESPONSE)))
        })
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
