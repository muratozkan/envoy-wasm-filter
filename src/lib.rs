use std::time::Duration;

use greeter::{Request1, Reply1};
use prost::Message;
use proxy_wasm::traits::{Context, HttpContext};
use proxy_wasm::types::{Action, LogLevel};

pub mod greeter {
    include!(concat!(env!("OUT_DIR"), "/multifiles.rs"));
}

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_http_context(|context_id, _| -> Box<dyn HttpContext> {
        Box::new(MyFilterContext { context_id })
    });
}

struct MyFilterContext {
    context_id: u32
}

impl Context for MyFilterContext {
    fn on_grpc_call_response(&mut self, token_id: u32, status_code: u32, response_size: usize) {
        log::info!("===> gRPC response for {}: {} ({} bytes)", token_id, status_code, response_size);

        let bytes = self.get_grpc_call_response_body(0, response_size)
            .expect("Expecting grpc response body");

        let token = Reply1::decode(bytes.as_slice())
            .expect("Can't understand grpc reply");

        self.set_http_request_header("token", None);
        self.add_http_request_header("my-token", token.message.as_str());
        self.resume_http_request();
    }
}

impl HttpContext for MyFilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        if let Some(token) = self.get_http_request_header("token") {
            log::info!("Found token header: #{} -> {}", self.context_id, &token);

            let mut req = Request1::default();
            req.name = token;

            let encoded = req.encode_to_vec();

            let result = self.dispatch_grpc_call("grpc_service", "multifiles.HelloService", "SayHello", Vec::new(), Some(&encoded), Duration::from_secs(10));
            log::info!("===> gRPC Dispatch: {:?}", result);
            return Action::Pause
        }

        log::info!("No token found, block request.");
        self.send_http_response(
            401,
            vec![("Rejected-By", "my-filter")],
            Some(b"Forbidden\n")
        );
        Action::Pause
    }
}
