use actix::prelude::*;
use image::EncodableLayout;
use inaturalist::models::Observation;
use std::{error, fmt, sync};

pub struct LoadImageMessage {
    pub observation: Box<Observation>,
}

impl Message for LoadImageMessage {
    type Result = ();
}

#[derive(Default)]
pub struct ImageStoreActor {
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
}

impl actix::Supervised for ImageStoreActor {}

impl SystemService for ImageStoreActor {}

impl Actor for ImageStoreActor {
    type Context = Context<Self>;
}

impl Handler<LoadImageMessage> for ImageStoreActor {
    type Result = ();

    fn handle(&mut self, msg: LoadImageMessage, ctx: &mut Self::Context) -> Self::Result {
        let image_store = self.image_store.clone();

        let c = async {
            if let Some(photo_url) = msg
                .observation
                .photos
                .as_ref()
                .and_then(|p| p.get(0).map(|p| p.url.to_owned()))
            {
                let image_url = photo_url.as_ref().unwrap().replace("square", "large");
                fetch_image(image_url, image_store, *msg.observation)
                    .await
                    .unwrap();
            }
        };

        ctx.spawn(Box::pin(c).into_actor(self));
    }
}

async fn fetch_image(
    url: String,
    image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
    observation: Observation,
) -> Result<(), Box<dyn error::Error>> {
    inaturalist_fetch::inaturalist_rate_limiter()
        .until_ready()
        .await;
    tracing::info!("Fetching image...");
    let response = reqwest::get(&url).await?;
    tracing::info!("Fetched image. Parsing response...");
    let retained_image = parse_image_response(response).await?;
    tracing::info!("Parsed response. Storing image...");
    image_store
        .write()
        .unwrap()
        .insert(observation.id.unwrap(), url, retained_image);
    tracing::info!("Stored iamge.");
    Ok(())
}

#[derive(Debug)]
enum ParseImageResponseError {
    NoHeader,
    NotAnImage,
}

impl fmt::Display for ParseImageResponseError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl error::Error for ParseImageResponseError {}

async fn parse_image_response(
    response: reqwest::Response,
) -> Result<egui_extras::RetainedImage, ParseImageResponseError> {
    match response.headers().get(reqwest::header::CONTENT_TYPE) {
        Some(header_value) => {
            if header_value.as_bytes().starts_with(b"image/") {
                Ok(egui_extras::RetainedImage::from_image_bytes(
                    response.url().as_str().to_owned(),
                    response.bytes().await.unwrap().as_bytes(),
                )
                .unwrap())
            } else {
                Err(ParseImageResponseError::NotAnImage)
            }
        }
        None => Err(ParseImageResponseError::NoHeader),
    }
}
