use log::debug;
use proxy_wasm::traits::{Context, HttpContext};
use proxy_wasm::types::{Action, LogLevel};

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_http_context(|context_id, _| -> Box<dyn HttpContext> {
        Box::new(MyFilterContext { context_id })
    });
}

struct MyFilterContext {
    context_id: u32,
}

impl Context for MyFilterContext {}

impl HttpContext for MyFilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        for (name, value) in &self.get_http_request_headers() {
            debug!("In Wasm: #{} -> {}: {}", self.context_id, name, value);
        }

        match self.get_http_request_header("token") {
            Some(token) if token.len() > 3 => {
                self.resume_http_request();
                Action::Continue
            }
            _ => {
                self.send_http_response(
                    403,
                    vec![("Rejected-By", "my-filter")],
                    Some(b"Forbidden\n"),
                );
                Action::Pause
            }
        }
    }
}
