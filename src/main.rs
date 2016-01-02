extern crate hyper;
extern crate hyperlocal;

use hyperlocal::{DomainUrl, SocketConnector};

fn main() {
    let client = hyper::Client::with_connector(hyperlocal::SocketConnector);
    let mut res = client.get(DomainUrl::new("/var/run/docker.sock", "/info")).send().unwrap();
    std::io::copy(&mut res, &mut std::io::stdout()).unwrap();
}
