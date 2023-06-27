use actix_web::{web, HttpResponse};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Error)]
pub enum ApplicationError {
    #[display(fmt = "An internal server error occurred")]
    InternalError,
}

impl actix_web::ResponseError for ApplicationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::plaintext())
            .body(self.to_string())
    }

    fn status_code(&self) -> reqwest::StatusCode {
        match *self {
            ApplicationError::InternalError => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Received TCP error\n{0}")]
    Tcp(reqwest::Error),
    #[error("Received HTTP error status code\n{0}")]
    HttpResponse(reqwest::Error),
    #[error("Error parsing JSON response\n{0}")]
    JsonParse(reqwest::Error),
}

pub async fn add_post(
    api_client: web::Data<ApiClient>,
    body: web::Json<NewPost>,
) -> Result<web::Json<Post>, ApplicationError> {
    info!("test log");

    match api_client.add(body).await {
        Ok(post) => Ok(web::Json(post)),
        Err(e) => {
            error!("{e}");
            Err(ApplicationError::InternalError)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    id: u64,
    title: String,
    body: String,
    user_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPost {
    title: String,
    body: String,
    user_id: u64,
}

#[async_trait::async_trait]
pub trait PostAdder {
    async fn add(&self, body: web::Json<NewPost>) -> Result<Post, ApiError>;
}

pub struct ApiClient {
    client: reqwest::Client,
    config: ApiConfig,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .pool_idle_timeout(systemstat::Duration::from_secs(60))
                .timeout(systemstat::Duration::from_secs(10))
                .build()
                // It is not clear how this Client builder could error but
                // theoretically this is unsafe
                .unwrap(),
            config: ApiConfig::new(),
        }
    }

    pub fn get_client(&self) -> reqwest::Client {
        self.client.clone()
    }
}

#[async_trait::async_trait]
impl PostAdder for ApiClient {
    async fn add(&self, body: web::Json<NewPost>) -> Result<Post, ApiError> {
        let url = format!("{}/posts", self.config.api_url.clone());

        Ok(self
            .get_client()
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApiError::Tcp(e))?
            .error_for_status()
            .map_err(|e| ApiError::HttpResponse(e))?
            .json::<Post>()
            .await
            .map_err(|e| ApiError::JsonParse(e))?)
    }
}

pub struct ApiConfig {
    pub api_url: String,
}

impl ApiConfig {
    pub fn new() -> Self {
        Self {
            api_url: std::env::var("API_URL")
                .unwrap_or("https://jsonplaceholder.typicode.com".to_string()),
        }
    }
}

#[cfg(test)]
mod add_post_integration_tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn happy_path_() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ApiClient::new()))
                .route("/", web::post().to(add_post)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({
                "userId": 1,
                "title": "new post who dis?",
                "body": "lorem ipsum blah, blah, blah"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
    }
}

#[cfg(test)]
mod add_post_unit_tests {
    use super::*;
    use actix_web::{test, web, App};

    #[test_log::test(actix_web::test)]
    async fn happy_path_() {
        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .wrap(actix_web::middleware::Logger::new("%a %{User-Agent}i"))
                .app_data(web::Data::new(FakeApiClient::new()))
                .route("/", web::post().to(add_post)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(serde_json::json!({
                "userId": 1,
                "title": "new post who dis?",
                "body": "lorem ipsum blah, blah, blah"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
    }

    pub struct FakeApiClient {}

    impl FakeApiClient {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait::async_trait]
    impl PostAdder for FakeApiClient {
        async fn add(&self, new_post: web::Json<NewPost>) -> Result<Post, ApiError> {
            Ok(make_future(Post {
                id: 101,
                title: new_post.title.clone(),
                body: new_post.body.clone(),
                user_id: new_post.user_id.clone(),
            })
            .await)
        }
    }

    fn make_future<T>(val: T) -> std::future::Ready<T> {
        std::future::ready(val)
    }
}
