use std::collections::HashMap;
use std::time::Duration;

use greeter::{HelloRequest, HelloReply, HiRequest, HiReply};
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
        Box::new(MyFilterContext { context_id, call_map: HashMap::new() })
    });
}

struct MyFilterContext {
    context_id: u32,
    call_map: HashMap<u32, String>
}

impl MyFilterContext {
    fn external_call_failed(&mut self, name: &str, status_code: u32) {
        self.add_http_request_header("my-filter-status", format!("{}; failed; {}", name, &status_code).as_str());
        self.resume_http_request();
    }
}

impl Context for MyFilterContext {

    fn on_grpc_call_response(&mut self, token_id: u32, status_code: u32, response_size: usize) {
        log::info!("===> gRPC response for {}: {} ({} bytes)", token_id, status_code, response_size);

        if let Some(service) =  self.call_map.remove(&token_id) {
            if status_code != 0 {
                self.external_call_failed(service.as_str(), status_code);
                return;
            }

            let bytes = self.get_grpc_call_response_body(0, response_size)
                .expect("Expecting grpc response body");

            let reply = match service.as_str() {
                "SayHello" => HelloReply::decode(bytes.as_slice()).map(|x| x.hello),
                "SayHi" => HiReply::decode(bytes.as_slice()).map(|x| x.hi),
                _ => Ok("unknown".to_string())
            };

            let token = reply
                .expect("Can't understand grpc reply");
    
            self.set_http_request_header("token", None);
            self.add_http_request_header("my-token", token.as_str());    
        } else {
            self.external_call_failed("unknown", status_code);
            return;
        }
        self.resume_http_request();
    }
}

impl HttpContext for MyFilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        if let Some(token) = self.get_http_request_header("token") {
            log::info!("Found header: token=#{} -> {}", self.context_id, &token);

            let req = HelloRequest {
                name: token
            };
            let encoded = req.encode_to_vec();

            let result = self.dispatch_grpc_call("grpc_service", "multifiles.HelloService", "SayHello", Vec::new(), Some(&encoded), Duration::from_secs(10));
            log::info!("===> gRPC Dispatch: HelloService/SayHello {:?}", result);
            if result.is_ok() {
                self.call_map.insert(result.ok().unwrap(), "SayHello".to_string());
                return Action::Pause
            }
        } else if let Some(other) = self.get_http_request_header("other") {
            log::info!("Found header: other=#{} -> {}", self.context_id, &other);

            let req = HiRequest {
                hi_name: other
            };
            let encoded = req.encode_to_vec();

            let result = self.dispatch_grpc_call("grpc_service", "multifiles.HelloService", "SayHi", Vec::new(), Some(&encoded), Duration::from_secs(10));
            log::info!("===> gRPC Dispatch: HelloService/SayHi {:?}", result);
            if result.is_ok() {
                self.call_map.insert(result.ok().unwrap(), "SayHi".to_string());
                return Action::Pause
            }
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
