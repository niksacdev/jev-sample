use std::{env, io, time::Duration};

#[tokio::main]
async fn main() -> io::Result<()> {
    let jev = match env::var("REASSURE_ASSESSOR") {
        Err(env::VarError::NotPresent) => None,
        Ok(value) if value == "rules" => None,
        Ok(value) if value == "jev" => {
            let key = env::var("TYPESAFE_API_KEY").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Jev mode requires TYPESAFE_API_KEY",
                )
            })?;
            Some(
                jev_sample::jev::Jev::new(
                    "https://api.typesafe.ai/v1/systemone".into(),
                    &key,
                    Duration::from_secs(15),
                )
                .map_err(io::Error::other)?,
            )
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "REASSURE_ASSESSOR must be rules or jev",
            ));
        }
    };
    eprintln!(
        "Synthetic local workspace; assessor={}; history is lost on restart.",
        if jev.is_some() {
            "jev"
        } else {
            "keyword_baseline"
        }
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("API listening on http://{}", listener.local_addr()?);
    axum::serve(
        listener,
        jev_sample::http::router_with(jev_sample::http::Application::new(jev)),
    )
    .await
}
