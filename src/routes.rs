use views;
use http::{Request, HTTPMethod};

pub fn route_request(request: &mut Request) -> (String, u32) {
    let view_fn: fn(&mut Request) -> (String, u32) = 
        match (&request.method, request.path.as_slice()) {
        (&HTTPMethod::GET, "/") => views::index, 
        (&HTTPMethod::GET, "/ws/") => views::ws, 
        _ => views::error_404,
    };

    view_fn(request)
}
