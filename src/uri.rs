use hyper::Uri as HyperUri;
use std::path::Path;

#[derive(Debug)]
pub struct Uri {
    hyper_uri: HyperUri,
}

impl Uri {
    pub fn new(socket: impl AsRef<Path>, path: &str) -> Self {
        let host = hex::encode(socket.as_ref().to_string_lossy().as_bytes());
        let host_str = format!("unix://{}:0{}", host, path);
        let hyper_uri: HyperUri = host_str.parse().unwrap();

        Self { hyper_uri }
    }
}

impl From<Uri> for HyperUri {
    fn from(uri: Uri) -> Self {
        uri.hyper_uri
    }
}
