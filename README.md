<div align="center">
<p><img alt="Savlo" width="132" style="max-width:40%;min-width:60px;" src="https://salvo.rs/images/logo-text.svg" /></p>
<p>
    <a href="https://github.com/salvo-rs/salvo/blob/main/README.md">English</a>&nbsp;&nbsp;
    <a href="https://github.com/salvo-rs/salvo/blob/main/README.zh-hans.md">简体中文</a>&nbsp;&nbsp;
    <a href="https://github.com/salvo-rs/salvo/blob/main/README.zh-hant.md">繁體中文</a>
</p>
<p>
<a href="https://github.com/salvo-rs/salvo/actions">
    <img alt="build status" src="https://github.com/salvo-rs/salvo/workflows/ci-linux/badge.svg?branch=main&event=push" />
</a>
<a href="https://github.com/salvo-rs/salvo/actions">
    <img alt="build status" src="https://github.com/salvo-rs/salvo/workflows/ci-macos/badge.svg?branch=main&event=push" />
</a>
<a href="https://github.com/salvo-rs/salvo/actions">
    <img alt="build status" src="https://github.com/salvo-rs/salvo/workflows/ci-windows/badge.svg?branch=main&event=push" />
</a>
<br>
<a href="https://crates.io/crates/salvo"><img alt="crates.io" src="https://img.shields.io/crates/v/salvo" /></a>
<a href="https://docs.rs/salvo"><img alt="Documentation" src="https://docs.rs/salvo/badge.svg" /></a>
<a href="https://github.com/rust-secure-code/safety-dance/"><img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-success.svg" /></a>
<a href="https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html"><img alt="Rust Version" src="https://img.shields.io/badge/rust-1.64%2B-blue" /></a>
<br>
<a href="https://salvo.rs">
    <img alt="Website" src="https://img.shields.io/badge/https-salvo.rs-%23f00" />
</a>
<a href="https://codecov.io/gh/salvo-rs/salvo"><img alt="codecov" src="https://codecov.io/gh/salvo-rs/salvo/branch/main/graph/badge.svg" /></a>
<a href="https://crates.io/crates/salvo"><img alt="Download" src="https://img.shields.io/crates/d/salvo.svg" /></a>
<img alt="License" src="https://img.shields.io/crates/l/salvo.svg" />
</p>
</div>

Salvo is an extremely simple and powerful Rust web backend framework. Only basic Rust knowledge is required to develop backend services.

> **Note**: salvo's [main](https://github.com/salvo-rs/salvo) branch is
> currently preparing breaking changes. For the most recently *released* code,
> look to the [0.37.x branch](https://github.com/salvo-rs/salvo/tree/v0.37.x).
> 
> 中国用户可以添加我微信(chrislearn), 拉微信讨论群.

## 🎯 Features
   - Built with [Hyper](https://crates.io/crates/hyper) and [Tokio](https://crates.io/crates/tokio);
   - Http1, Http2 and **Http3**;
   - Unified middleware and handle interface;
   - Limitless routers nesting;
   - Every router can have one or many middlewares;
   - Integrated Multipart form processing;
   - Support WebSocket;
   - Acme support, automatically get TLS certificate from [let's encrypt](https://letsencrypt.org/).

## ⚡️ Quick start
You can view samples [here](https://github.com/salvo-rs/salvo/tree/main/examples), or view [offical website](https://salvo.rs).

Create a new rust project:

```bash
cargo new hello_salvo --bin
```

Add this to `Cargo.toml`

```toml
[dependencies]
salvo = "*"
tokio = { version = "1", features = ["macros"] }
```

Create a simple function handler in the main.rs file, we call it `hello`, this function just render plain text `"Hello World"`.

```rust
use salvo::prelude::*;

#[handler]
async fn hello(res: &mut Response) {
    res.render(Text::Plain("Hello World"));
}
```

In the `main` function, we need to create a root Router first, and then create a server and call it's `bind` function:

```rust
use salvo::prelude::*;

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}
#[tokio::main]
async fn main() {
    let router = Router::new().get(hello);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
```

### Middleware
There is no difference between Handler and Middleware, Middleware is just Handler. **So you can write middlewares without to know concpets like associated type, generic type. You can write middleware if you can write function!!!**

```rust
use salvo::http::header::{self, HeaderValue};
use salvo::prelude::*;

#[handler]
async fn add_header(res: &mut Response) {
    res.headers_mut()
        .insert(header::SERVER, HeaderValue::from_static("Salvo"));
}
```

Then add it to router:

```rust
Router::new().hoop(add_header).get(hello)
```

This is a very simple middleware, it add `Header` to `Response`, view [full source code](https://github.com/salvo-rs/salvo/blob/main/examples/middleware-add-header/src/main.rs).

### Chainable tree routing system

Normally we write routing like this：

```rust
Router::with_path("articles").get(list_articles).post(create_article);
Router::with_path("articles/<id>")
    .get(show_article)
    .patch(edit_article)
    .delete(delete_article);
```

Often viewing articles and article lists does not require user login, but creating, editing, deleting articles, etc. require user login authentication permissions. The tree-like routing system in Salvo can meet this demand. We can write routers without user login together:

```rust
Router::with_path("articles")
    .get(list_articles)
    .push(Router::with_path("<id>").get(show_article));
```

Then write the routers that require the user to login together, and use the corresponding middleware to verify whether the user is logged in:
```rust
Router::with_path("articles")
    .hoop(auth_check)
    .post(list_articles)
    .push(Router::with_path("<id>").patch(edit_article).delete(delete_article));
```

Although these two routes have the same `path("articles")`, they can still be added to the same parent route at the same time, so the final route looks like this:

```rust
Router::new()
    .push(
        Router::with_path("articles")
            .get(list_articles)
            .push(Router::with_path("<id>").get(show_article)),
    )
    .push(
        Router::with_path("articles")
            .hoop(auth_check)
            .post(list_articles)
            .push(Router::with_path("<id>").patch(edit_article).delete(delete_article)),
    );
```

`<id>` matches a fragment in the path, under normal circumstances, the article `id` is just a number, which we can use regular expressions to restrict `id` matching rules, `r"<id:/\d+/>"`.

You can also use `<*>` or `<**>` to match all remaining path fragments. In order to make the code more readable, you can also add appropriate name to make the path semantics more clear, for example: `<**file_path>`.

Some regular expressions for matching paths need to be used frequently, and it can be registered in advance, such as GUID:

```rust
PathFilter::register_wisp_regex(
    "guid",
    Regex::new("[0-9a-fA-F]{8}-([0-9a-fA-F]{4}-){3}[0-9a-fA-F]{12}").unwrap(),
);
```

This makes it more concise when path matching is required:

```rust
Router::with_path("<id:guid>").get(index)
```

View [full source code](https://github.com/salvo-rs/salvo/blob/main/examples/routing-guid/src/main.rs)

### File upload
We can get file async by the function `file` in `Request`:

```rust
#[handler]
async fn upload(req: &mut Request, res: &mut Response) {
    let file = req.file("file").await;
    if let Some(file) = file {
        let dest = format!("temp/{}", file.name().unwrap_or_else(|| "file".into()));
        if let Err(e) = tokio::fs::copy(&file.path, Path::new(&dest)).await {
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        } else {
            res.render("Ok");
        }
    } else {
        res.set_status_code(StatusCode::BAD_REQUEST);
    }
}
```

### Extract data from request

You can easily get data from multiple different data sources and assemble it into the type you want. You can define a custom type first, for example:

```rust
#[derive(Serialize, Deserialize, Extractible, Debug)]
/// Get the data field value from the body by default.
#[extract(default_source(from = "body"))]
struct GoodMan<'a> {
    /// The id number is obtained from the request path parameter, and the data is automatically parsed as i64 type.
    #[extract(source(from = "param"))]
    id: i64,
    /// Reference types can be used to avoid memory copying.
    username: &'a str,
    first_name: String,
    last_name: String,
}
```

Then in `Handler` you can get the data like this:

```rust
#[handler]
async fn edit(req: &mut Request) {
    let good_man: GoodMan<'_> = req.extract().await.unwrap();
}
```

You can even pass the type directly to the function as a parameter, like this:

```rust
#[handler]
async fn edit<'a>(good_man: GoodMan<'a>) {
    res.render(Json(good_man));
}
```

There is considerable flexibility in the definition of data types, and can even be resolved into nested structures as needed:

```rust
#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct GoodMan<'a> {
    #[extract(source(from = "param"))]
    id: i64,
    #[extract(source(from = "query"))]
    username: &'a str,
    first_name: String,
    last_name: String,
    lovers: Vec<String>,
    /// The nested field is completely reparsed from Request.
    #[extract(source(from = "request"))]
    nested: Nested<'a>,
}

#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct Nested<'a> {
    #[extract(source(from = "param"))]
    id: i64,
    #[extract(source(from = "query"))]
    username: &'a str,
    first_name: String,
    last_name: String,
    #[extract(rename = "lovers")]
    #[serde(default)]
    pets: Vec<String>,
}
```

View [full source code](https://github.com/salvo-rs/salvo/blob/main/examples/extract-nested/src/main.rs)

### More Examples
Your can find more examples in [examples](./examples/) folder. You can run these examples with the following command:

```
cargo run --bin example-basic-auth
```

You can use any example name you want to run instead of `basic-auth` here.

## 🚀 Performance
Benchmark testing result can be found from here:

[https://web-frameworks-benchmark.netlify.app/result?l=rust](https://web-frameworks-benchmark.netlify.app/result?l=rust)

[https://www.techempower.com/benchmarks/#section=data-r21](https://www.techempower.com/benchmarks/#section=data-r21)
![techempower](assets/tp.jpg)


## 🎇 Deployment

You can deploy your salvo projects through [shuttle.rs](https://www.shuttle.rs/), it is very easy, you can read shuttle's offical document [here](https://docs.shuttle.rs/guide/salvo-examples.html).

## 🩸 Contributing

Contributions are absolutely, positively welcome and encouraged! Contributions come in many forms. You could:

  - Submit a feature request or bug report as an issue;
  - Comment on issues that require feedback;
  - Contribute code via pull requests;
  - Publish Salvo-related technical articles on blogs or technical platforms。

All pull requests are code reviewed and tested by the CI. Note that unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Salvo by you shall be dual licensed under the MIT License, without any additional terms or conditions.

## ☕ Supporters

Salvo is an open source project. If you want to support Salvo, you can ☕ [**buy a coffee here**](https://ko-fi.com/chrislearn).
<p style="text-align: center;">
<img src="https://salvo.rs/images/alipay.png" alt="Alipay" width="180"/>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<img src="https://salvo.rs/images/weixin.png" alt="Weixin" width="180"/>
</p>

## ⚠️ License

Salvo is licensed under either of
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
