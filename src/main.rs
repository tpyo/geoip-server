#[macro_use]
extern crate serde_derive;
extern crate maxminddb;
extern crate futures;
extern crate futures_cpupool;
extern crate serde;
extern crate serde_json;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;
extern crate num_cpus;

use std::sync::Arc;
use std::io;
use std::net::{IpAddr, SocketAddr, AddrParseError};
use std::str::FromStr;
use std::collections::BTreeMap;
use std::env::args;
use futures::{BoxFuture, Future};
use futures_cpupool::CpuPool;
use tokio_minihttp::{Request, Response};
use tokio_proto::TcpServer;
use tokio_service::Service;
use maxminddb::geoip2;
use maxminddb::geoip2::model::{City, Country, Subdivision, Location};

#[derive(Serialize)]
struct Result {
    ip: String,
    latitude: f64,
    longitude: f64,
    time_zone: String,
    iso_code: String,
    city: BTreeMap<String, String>,
    subdivisions: Vec<BTreeMap<String, String>>, 
    country: BTreeMap<String, String>,
    registered_country: BTreeMap<String, String>
}

struct Server {
    thread_pool: CpuPool,
    reader: Arc<maxminddb::Reader>
}

fn get_city_names(city: &Option<City>) -> BTreeMap<String, String> {
	match *city {
		Some(ref xcity) => xcity.names.clone().unwrap(),
		None => std::collections::BTreeMap::new()
	}
}

fn get_country_names(country: &Option<Country>) -> BTreeMap<String, String> {
	match *country {
		Some(ref xcountry) => xcountry.names.clone().unwrap(),
		None => std::collections::BTreeMap::new()
	}
}

fn get_country_iso_code(country: &Option<Country>) -> String {
	match *country {
		Some(ref xcountry) => xcountry.iso_code.clone().unwrap(),
		None => String::from("")
	}
}

fn get_location_latitude(location: &Option<Location>) -> f64 {
	match *location {
		Some(ref xlocation) => xlocation.clone().latitude.unwrap(),
		None => 0.0
	}
}

fn get_location_longitude(location: &Option<Location>) -> f64 {
	match *location {
		Some(ref xlocation) => xlocation.clone().longitude.unwrap(),
		None => 0.0
	}
}

fn get_location_time_zone(location: &Option<Location>) -> String {
	match *location {
		Some(ref xlocation) => xlocation.clone().time_zone.unwrap(),
		None => String::from("")
	}
}

fn get_subdivision_names(subdivisions: &Option<Vec<Subdivision>>) -> Vec<BTreeMap<String, String>> {
	match *subdivisions {
		Some(ref xsubdivisions) => {
			let mut subdivisions = vec![];
			for subdivision in xsubdivisions {
				subdivisions.push(subdivision.names.clone().unwrap());
			}
			subdivisions
		}
		None => Vec::new()
	}
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = BoxFuture<Response, io::Error>;

    fn call(&self, req: Request) -> Self::Future {
       
        let db = self.reader.clone();
        let msg = self.thread_pool.spawn_fn(move || {
			let path = req.path();
        	let foo = &path[1..];
        	//println!("Looking up {}", foo);

			match IpAddr::from_str(foo) {
			    Ok(ip) => {

			    	match db.lookup(ip) {
			    		Ok(xlookup) => {
			    			let lookup: geoip2::City = xlookup;
			    			let result = Result {
				                ip: String::from_str(foo).unwrap(),
				                longitude: get_location_longitude(&lookup.location),
				                latitude: get_location_latitude(&lookup.location),
				                time_zone: get_location_time_zone(&lookup.location),
				                iso_code: get_country_iso_code(&lookup.country),
				                city: get_city_names(&lookup.city),
				                subdivisions: get_subdivision_names(&lookup.subdivisions),
				                country: get_country_names(&lookup.country),
				                registered_country: get_country_names(&lookup.registered_country),
				            };
				            Ok(serde_json::to_string(&result).unwrap())
			    		},
			    		Err(_) => Ok(String::from("{\"error\": \"not_found\"}"))
			    	}
			    },
			    Err(AddrParseError(_)) => Ok(String::from("{\"error\": \"invalid_address\"}")),
			}
        });

        msg.map(|body| {
            let mut response = Response::new();
            response.header("Content-Type", "application/json; charset=UTF-8");
            response.body(&body);
            response
        }).boxed()
    }
}

fn main() {
    let args = args().collect::<Vec<_>>();

    if args.len() < 3 {
        println!("Usage: {} <bind ip:port> <mmdb file>", args[0]);
        return;
    }

	let addr = match SocketAddr::from_str(&args[1]) {
	    Ok(addr) => addr,
	    Err(AddrParseError(_)) => {
	    	println!("Error: Invalid bind address: \"{}\".", &args[1]);
	    	return
	    }
	};

    let reader = match maxminddb::Reader::open(&args[2]) {
    	Ok(reader) => reader,
        Err(_) => {
        	println!("Error: Unable to open database file: \"{}\"", &args[2]);
        	return
        }
    };

    let db = Arc::new(reader);
    let thread_pool = CpuPool::new(num_cpus::get());
    TcpServer::new(tokio_minihttp::Http, addr).serve(move || {
        Ok(Server {
            thread_pool: thread_pool.clone(),
            reader: db.clone(),
        })
    });
}
