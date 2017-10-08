# 0.4.1

* implement Clone for `UnixConnector` [@letmutx](https://github.com/softprops/hyperlocal/pull/7)

# 0.4.0

* refactor for async hyper
* `hyperlocal::DomainUrl` is now `hyperlocal::Uri` the semantics are the same but the name now matches hyper's new name can can be lifted into hypers type

```rust
let uri: hyper:Uri =
   hyperlocal::Uri(
     "path/to/server.sock",
     "/foo/bar?baz=boom"
   ).into();
```
* `hyperlocal::UnitSocketConnector` is now just `hyperlocal::UnixConnector` to be more inline with the naming conventions behind`hyper::HttpConnector` and `hyper_tls::HttpsConnector`
* `hyperlocal::UnixSocketServer` is now  `hyperlocal::server::Http` to be more inline with hyper naming conventions

# 0.3.0

* enable using unix_socket from stdlib. [#4](https://github.com/softprops/hyperlocal/pull/4)
* upgrade to hyper 0.10

# 0.2.0

* upgraded to hyper 0.9 and transitively url 1.0


# 0.1.0

Initial release
