extern crate hyper;
extern crate hyperlocal;

use hyper::Client;
use hyperlocal::{DomainUrl, UnixConnector};

fn main() {
    let path = "test.sock";
    let client = Client::with_connector(UnixConnector);
    let mut res = client.get(DomainUrl::new(path, "/")).send().unwrap();
    std::io::copy(&mut res, &mut std::io::stdout()).unwrap();
}
