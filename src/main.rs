use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::Error;
use hyper::{service::service_fn, Response};
use hyper::header::CONTENT_TYPE;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use maxminddb::{geoip2, Mmap};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::env;
use tokio::net::TcpListener;


fn parse_args() -> Result<(SocketAddr, String), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        
        eprintln!("Usage: {} <bind ip:port> <mmdb file>", args[0]);
        std::process::exit(1);
    }

    let bind_host = &args[1];
    let addr = bind_host.to_socket_addrs()?.next().ok_or("Invalid bind host")?;
    let mmdb_file = args[2].clone();

    Ok((addr, mmdb_file))
}

async fn run_server(listener: TcpListener, db: Arc<maxminddb::Reader<Mmap>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening on http://{}", listener.local_addr()?);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let db = db.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let db = db.clone();
                let path = req.uri().path().trim_start_matches('/').to_string();

                async move {
                    match path.as_str() {
                        "healthz" => {
                            handle_healthcheck().await
                        },
                        _ => {
                            handle_request(&path, db).await
                        }
                    }
                }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_healthcheck() -> Result<Response<Full<Bytes>>, Error> {
    Ok(Response::builder()
    .status(StatusCode::OK)
    .body(Full::new(Bytes::from("{\"status\": \"healthy\"}"))).unwrap())
}

async fn handle_request(path: &str, db: Arc<maxminddb::Reader<Mmap>>) -> Result<Response<Full<Bytes>>, Error> {
    let body: Full<Bytes>;
    let status: StatusCode;

    if let Ok(ipaddr) = path.parse::<IpAddr>() {
        match db.lookup::<geoip2::City>(ipaddr) {
            Ok(lookup) => {
                match lookup {
                    Some(_) => {
                        status = StatusCode::OK;
                        body = Full::new(Bytes::from(
                            serde_json::to_string(&lookup).unwrap().to_string(),
                        ));
                    },
                    None => {
                        return Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Full::new(Bytes::from("{\"error\": \"not_found\"}"))).unwrap());
                    }
                }
                if lookup.is_none() {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from("{\"error\": \"not_found\"}"))).unwrap());
                }
            },
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(Bytes::from("{\"error\": \"internal_error\"}"))).unwrap());
            }
        }
    } else {
        status = StatusCode::BAD_REQUEST;
        body = Full::new(Bytes::from("{\"error\": \"invalid_ip\"}"))
    }

    Ok::<_, Error>(Response::builder()
    .header(CONTENT_TYPE, "application/json")
    .status(status)
    .body(body).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (addr, mmdb_file) = parse_args()?;

    let listener = TcpListener::bind(addr).await?;

    let db = Arc::new(maxminddb::Reader::open_mmap(mmdb_file)?);

    run_server(listener, db).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tokio_test::assert_ok;

    async fn mock_handle_request(ip: &str) -> Result<Response<Full<Bytes>>, Error> {
        let db = Arc::new(maxminddb::Reader::open_mmap("MaxMind-DB/test-data/GeoIP2-City-Test.mmdb").unwrap());
        handle_request(ip, db).await
    }

    #[tokio::test]
    async fn test_handle_request_with_valid_ip() {
        let ip = "89.160.20.128";
        let response = mock_handle_request(ip).await;
        assert_ok!(&response);
        let mut data = response.unwrap();
        assert_eq!(data.headers().get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(data.status(), StatusCode::OK);
        let body = data.frame().await.unwrap().unwrap().into_data().unwrap();
        let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
        assert_eq!(json["city"]["names"]["en"], "Linköping");
    }

    #[tokio::test]
    async fn test_handle_request_with_unknown_ip() {
        let ip = "192.168.0.1";
        let response = mock_handle_request(ip).await;
        assert_ok!(&response);
        let data = response.unwrap();
        assert_eq!(data.headers().get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(data.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_request_with_invalid_ip() {
        let ip = "invalid-ip";
        let response = mock_handle_request(ip).await;
        assert_ok!(&response);
        let data = response.unwrap();
        assert_eq!(data.status(), StatusCode::BAD_REQUEST);
    }
}