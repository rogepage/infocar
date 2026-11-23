use crate::http::state::AppState;
use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct WindowState {
    window_start: Instant,
    count: u32,
}

#[derive(Debug)]
pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    state: Mutex<WindowState>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            state: Mutex::new(WindowState {
                window_start: Instant::now(),
                count: 0,
            }),
        }
    }

    pub fn allow(&self) -> bool {
        let mut guard = self.state.lock().unwrap();
        let now = Instant::now();
        if now.duration_since(guard.window_start) >= self.window {
            guard.window_start = now;
            guard.count = 0;
        }
        if guard.count >= self.max_requests {
            return false;
        }
        guard.count += 1;
        true
    }
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.rate_limiter.allow() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "too_many_requests" })),
        )
            .into_response();
    }
    next.run(req).await
}
