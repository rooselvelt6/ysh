use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};

pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    let _ = headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    let _ = headers.insert("x-frame-options", "DENY".parse().unwrap());
    let _ = headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());
    let _ = headers.insert(
        "strict-transport-security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    let _ = headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    let _ = headers.insert(
        "permissions-policy",
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );
    let _ = headers.insert("cache-control", "no-store".parse().unwrap());
    let _ = headers.insert("x-permitted-cross-domain-policies", "none".parse().unwrap());

    response
}
