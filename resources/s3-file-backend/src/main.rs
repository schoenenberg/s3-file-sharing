use lambda_http::{
    handler,
    lambda_runtime::{self, Context, Error},
    IntoResponse, Request, Response,
};
use rusoto_s3::util::PreSignedRequest;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(func)).await?;
    Ok(())
}

#[tracing::instrument]
async fn func(event: Request, _ctx: Context) -> Result<impl IntoResponse, Error> {
    // Reading environment variables for bucket and aws credentials
    let bucket = std::env::var("BUCKET")?;
    let aws_key = std::env::var("AWS_ACCESS_KEY_ID")?;
    let aws_secret = std::env::var("AWS_SECRET_ACCESS_KEY")?;
    let aws_session_token = match std::env::var("AWS_SESSION_TOKEN") {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    // Set the default region
    let region = rusoto_core::Region::default();
    // Create AWS credentials
    let credentials = rusoto_core::credential::AwsCredentials::new(
        aws_key,
        aws_secret,
        aws_session_token,
        None,
    );

    // Retrieve the requested object key
    let uri = event.uri();
    let key = uri.path()
        .rsplit_once('/')
        .expect("at least one slash should be in there")
        .1
        .to_string();
    tracing::trace!(%uri, %key);

    let url = tracing::trace_span!("create presigned url")
        .in_scope(|| {
            // Create the request
            let req = rusoto_s3::GetObjectRequest {
                key,
                bucket,
                ..Default::default()
            };
            // Setting the expiration time for the url
            let option = rusoto_s3::util::PreSignedRequestOption {
                expires_in: std::time::Duration::from_secs(60),
            };
            // Generate the pre-signed URL
            req.get_presigned_url(&region, &credentials, &option)
        });

    Ok(Response::builder()
        .status(303)
        .header("Location", url.as_str())
        .body(())
        .unwrap())
}
