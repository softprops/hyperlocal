extern crate hyper;
extern crate hyperlocal;

use hyper::server::{Request, Response};
use hyperlocal::UnixSocketServer;

fn main() {
    let path = "test.sock";
    let server = UnixSocketServer::new(path).unwrap();
    server.handle(|_: Request, res: Response| {
        let _ = res.send(b"It's a Unix system. I know this.");
    }).unwrap();
    println!("listening @ {}", path);

}
