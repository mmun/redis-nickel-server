// TODO:
// - DRY up the 404 responses
// - Use &str instead of String where possible
// - Allow host/port configuration
// - Allow redis configuration
// - Support multiple apps via the Host header
// - Deploy to heroku

#[macro_use] extern crate nickel;
extern crate redis;
extern crate time;

use std::sync::Mutex;
use nickel::{Nickel, HttpRouter, QueryString};
use nickel::status::StatusCode;
use redis::{Commands, Connection, RedisResult};

fn main() {
    let redis_client = redis::Client::open("redis://localhost").unwrap();
    let redis_conn = redis_client.get_connection().unwrap();

    let redis_conn_lock = Mutex::new(redis_conn);

    let mut server = Nickel::new();

    server.utilize(middleware! { |request|
        println!(
            "{} {} {} {}",
            time::get_time().sec, 
            request.origin.remote_addr,
            request.origin.method,
            request.origin.uri
        );
    });

    server.get("/favicon.ico", middleware! { |_, mut response|
        response.set(StatusCode::NotFound);
        "<h1>Not Found</h1>"
    });

    server.get("**", middleware! { |request, mut response|
        fetch_index(&redis_conn_lock, request.query().get("v")).unwrap_or_else(|_err| {
            response.set(StatusCode::NotFound);
            "<h1>Not Found</h1>".to_string()
        })
    });

    server.listen("0.0.0.0:6767");
}

fn fetch_index(redis_conn_lock: &Mutex<redis::Connection>, version_option: Option<&str>) -> redis::RedisResult<String> {
    let redis_conn = &redis_conn_lock.lock().unwrap();

    let key = match version_option {
        None | Some("current") => try!(redis_conn.get("my-app:current")),
        Some(version)          => format!("my-app:{}", version),
    };

    redis_conn.get(key)
}
