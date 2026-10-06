use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("API listening on http://{}", listener.local_addr()?);
    axum::serve(listener, jev_sample::http::router()).await
}
