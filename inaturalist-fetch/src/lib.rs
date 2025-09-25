use geo_ext::Halve;
use inaturalist::apis::configuration::ApiKey;
use std::future::Future;
use std::{fmt, pin, sync};

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

pub fn get_inaturalist_request_config(
    api_token: &str,
) -> inaturalist::apis::configuration::Configuration {
    inaturalist::apis::configuration::Configuration {
        base_path: String::from("https://api.inaturalist.org/v1"),
        api_key: Some(ApiKey {
            prefix: None,
            key: api_token.to_string(),
        }),
        ..Default::default()
    }
}

use std::sync::OnceLock;

static INATURALIST_RATE_LIMIT_AMOUNT_CELL: OnceLock<governor::Quota> = OnceLock::new();
pub fn inaturalist_rate_limit_amount() -> &'static governor::Quota {
    INATURALIST_RATE_LIMIT_AMOUNT_CELL
        .get_or_init(|| governor::Quota::with_period(std::time::Duration::from_secs(2)).unwrap())
}

type RateLimiter = governor::RateLimiter<
    governor::state::direct::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
>;

static INATURALIST_RATE_LIMITER_CELL: OnceLock<RateLimiter> = OnceLock::new();
pub fn inaturalist_rate_limiter() -> &'static RateLimiter {
    INATURALIST_RATE_LIMITER_CELL
        .get_or_init(|| governor::RateLimiter::direct(*inaturalist_rate_limit_amount()))
}

#[derive(Copy, Clone, Debug)]
pub struct SubdividedRect(pub crate::Rect);

/// iNaturalist will not let us page past 10,000 results.
const MAX_RESULTS: i32 = 10_000;

const MAX_RESULTS_PER_PAGE: u32 = 200;

pub fn subdivide_rect_iter(
    rect: Rect,
    mut request: inaturalist::apis::observations_api::ObservationsGetParams,
    api_token: String,
) -> genawaiter::sync::Gen<
    Result<
        SubdividedRect,
        inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
    >,
    (),
    pin::Pin<Box<dyn Future<Output = ()> + Send>>,
> {
    request.only_id = Some(true);
    tracing::info!("rect size: {:?}", rect);

    genawaiter::sync::Gen::new_boxed(|co| async move {
        let page = 1;
        let per_page = 1;
        inaturalist_rate_limiter().until_ready().await;

        let response = match inaturalist::apis::observations_api::observations_get(
            &get_inaturalist_request_config(&api_token),
            merge_params(build_params(rect, page, per_page), request.clone()),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                co.yield_(Err(e)).await;
                return;
            }
        };

        if response.total_results.unwrap() < MAX_RESULTS {
            tracing::info!("Rect is sufficient");
            co.yield_(Ok(SubdividedRect(rect))).await;
            return;
        }

        tracing::info!(
            "Splitting rect (total_results: {})",
            response.total_results.unwrap()
        );
        let (rect1, rect2) = rect.halve();

        let mut gen = subdivide_rect_iter(rect1, request.clone(), api_token.clone());
        while let genawaiter::GeneratorState::Yielded(n) = gen.async_resume().await {
            tracing::info!("Yield rect1");
            co.yield_(n).await;
        }

        let mut gen = subdivide_rect_iter(rect2, request.clone(), api_token);
        while let genawaiter::GeneratorState::Yielded(n) = gen.async_resume().await {
            tracing::info!("Yield rect1");
            co.yield_(n).await;
        }
    })
}

pub async fn fetch(
    rect: Rect,
    on_observation: impl Fn(inaturalist::models::Observation),
    soft_limit: &sync::atomic::AtomicI32,
    request: inaturalist::apis::observations_api::ObservationsGetParams,
    api_token: &str,
) -> Result<(), inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>>
{
    let per_page = MAX_RESULTS_PER_PAGE;

    for page in 1.. {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            break;
        }

        tracing::info!("Fetching observations...");
        inaturalist_rate_limiter().until_ready().await;
        let response = inaturalist::apis::observations_api::observations_get(
            &get_inaturalist_request_config(api_token),
            merge_params(request.clone(), build_params(rect, page, per_page)),
        )
        .await?;
        tracing::info!("Fetched {} observations...", response.results.len());

        soft_limit.fetch_sub(
            response.results.len() as i32,
            sync::atomic::Ordering::Relaxed,
        );
        for result in response.results {
            tracing::info!("ON OBSERVATION");
            on_observation(result);
        }

        let per_page = response.per_page.unwrap() as u32;
        let total_results = response.total_results.unwrap() as u32;

        let last_page: u32 = 1 + total_results / per_page;

        if page == last_page {
            tracing::info!(
                "No more pages (total results: {})",
                response.total_results.unwrap()
            );
            break;
        } else {
            tracing::info!(
                "New page ({} / {} | {})",
                per_page * page,
                total_results,
                (per_page as f32 * page as f32) / (total_results as f32)
            );
        }
    }

    Ok(())
}

fn build_params(
    rect: Rect,
    page: u32,
    per_page: u32,
) -> inaturalist::apis::observations_api::ObservationsGetParams {
    inaturalist::apis::observations_api::ObservationsGetParams {
        swlat: Some(*rect.min().y),
        swlng: Some(*rect.min().x),
        nelat: Some(*rect.max().y),
        nelng: Some(*rect.max().x),
        // captive: Some(false),
        // identified: Some(true),
        // identifications: Some(String::from("most_agree")),
        // native: Some(true),
        order: Some("asc".to_string()),
        order_by: Some("observed_on".to_string()),
        per_page: Some(per_page.to_string()),
        page: Some(page.to_string()),
        acc: None,
        captive: None,
        endemic: None,
        geo: None,
        id_please: None,
        identified: None,
        introduced: None,
        mappable: None,
        native: None,
        out_of_range: None,
        pcid: None,
        photos: None,
        popular: None,
        sounds: None,
        taxon_is_active: None,
        threatened: None,
        verifiable: None,
        licensed: None,
        photo_licensed: None,
        expected_nearby: None,
        id: None,
        not_id: None,
        license: None,
        ofv_datatype: None,
        photo_license: None,
        place_id: None,
        project_id: None,
        rank: None,
        site_id: None,
        sound_license: None,
        taxon_id: None,
        without_taxon_id: None,
        taxon_name: None,
        user_id: None,
        user_login: None,
        ident_user_id: None,
        hour: None,
        day: None,
        month: None,
        year: None,
        created_day: None,
        created_month: None,
        created_year: None,
        term_id: None,
        term_value_id: None,
        without_term_id: None,
        without_term_value_id: None,
        term_id_or_unknown: None,
        annotation_user_id: None,
        acc_above: None,
        acc_below: None,
        acc_below_or_unknown: None,
        d1: None,
        d2: None,
        created_d1: None,
        created_d2: None,
        created_on: None,
        observed_on: None,
        unobserved_by_user_id: None,
        apply_project_rules_for: None,
        cs: None,
        csa: None,
        csi: None,
        geoprivacy: None,
        taxon_geoprivacy: None,
        obscuration: None,
        hrank: None,
        lrank: None,
        iconic_taxa: None,
        id_above: None,
        id_below: None,
        identifications: None,
        lat: None,
        lng: None,
        radius: None,
        list_id: None,
        not_in_project: None,
        not_matching_project_rules_for: None,
        observation_accuracy_experiment_id: None,
        q: None,
        search_on: None,
        quality_grade: None,
        updated_since: None,
        viewer_id: None,
        reviewed: None,
        locale: None,
        preferred_place_id: None,
        ttl: None,
        only_id: None,
    }
}

fn merge_params(
    params1: inaturalist::apis::observations_api::ObservationsGetParams,
    params2: inaturalist::apis::observations_api::ObservationsGetParams,
) -> inaturalist::apis::observations_api::ObservationsGetParams {
    inaturalist::apis::observations_api::ObservationsGetParams {
        acc: params1.acc.or(params2.acc),
        captive: params1.captive.or(params2.captive),
        endemic: params1.endemic.or(params2.endemic),
        geo: params1.geo.or(params2.geo),
        id_please: params1.id_please.or(params2.id_please),
        identified: params1.identified.or(params2.identified),
        introduced: params1.introduced.or(params2.introduced),
        mappable: params1.mappable.or(params2.mappable),
        native: params1.native.or(params2.native),
        out_of_range: params1.out_of_range.or(params2.out_of_range),
        pcid: params1.pcid.or(params2.pcid),
        photos: params1.photos.or(params2.photos),
        popular: params1.popular.or(params2.popular),
        sounds: params1.sounds.or(params2.sounds),
        taxon_is_active: params1.taxon_is_active.or(params2.taxon_is_active),
        threatened: params1.threatened.or(params2.threatened),
        verifiable: params1.verifiable.or(params2.verifiable),
        licensed: params1.licensed.or(params2.licensed),
        photo_licensed: params1.photo_licensed.or(params2.photo_licensed),
        id: params1.id.or(params2.id),
        not_id: params1.not_id.or(params2.not_id),
        license: params1.license.or(params2.license),
        ofv_datatype: params1.ofv_datatype.or(params2.ofv_datatype),
        photo_license: params1.photo_license.or(params2.photo_license),
        place_id: params1.place_id.or(params2.place_id),
        project_id: params1.project_id.or(params2.project_id),
        rank: params1.rank.or(params2.rank),
        site_id: params1.site_id.or(params2.site_id),
        sound_license: params1.sound_license.or(params2.sound_license),
        taxon_id: params1.taxon_id.or(params2.taxon_id),
        without_taxon_id: params1.without_taxon_id.or(params2.without_taxon_id),
        taxon_name: params1.taxon_name.or(params2.taxon_name),
        user_id: params1.user_id.or(params2.user_id),
        user_login: params1.user_login.or(params2.user_login),
        ident_user_id: params1.ident_user_id.or(params2.ident_user_id),
        day: params1.day.or(params2.day),
        month: params1.month.or(params2.month),
        year: params1.year.or(params2.year),
        term_id: params1.term_id.or(params2.term_id),
        term_value_id: params1.term_value_id.or(params2.term_value_id),
        without_term_id: params1.without_term_id.or(params2.without_term_id),
        without_term_value_id: params1
            .without_term_value_id
            .or(params2.without_term_value_id),
        acc_above: params1.acc_above.or(params2.acc_above),
        acc_below: params1.acc_below.or(params2.acc_below),
        acc_below_or_unknown: params1
            .acc_below_or_unknown
            .or(params2.acc_below_or_unknown),
        d1: params1.d1.or(params2.d1),
        d2: params1.d2.or(params2.d2),
        created_d1: params1.created_d1.or(params2.created_d1),
        created_d2: params1.created_d2.or(params2.created_d2),
        created_on: params1.created_on.or(params2.created_on),
        observed_on: params1.observed_on.or(params2.observed_on),
        unobserved_by_user_id: params1
            .unobserved_by_user_id
            .or(params2.unobserved_by_user_id),
        apply_project_rules_for: params1
            .apply_project_rules_for
            .or(params2.apply_project_rules_for),
        cs: params1.cs.or(params2.cs),
        csa: params1.csa.or(params2.csa),
        csi: params1.csi.or(params2.csi),
        geoprivacy: params1.geoprivacy.or(params2.geoprivacy),
        taxon_geoprivacy: params1.taxon_geoprivacy.or(params2.taxon_geoprivacy),
        hrank: params1.hrank.or(params2.hrank),
        lrank: params1.lrank.or(params2.lrank),
        iconic_taxa: params1.iconic_taxa.or(params2.iconic_taxa),
        id_above: params1.id_above.or(params2.id_above),
        id_below: params1.id_below.or(params2.id_below),
        identifications: params1.identifications.or(params2.identifications),
        lat: params1.lat.or(params2.lat),
        lng: params1.lng.or(params2.lng),
        radius: params1.radius.or(params2.radius),
        nelat: params1.nelat.or(params2.nelat),
        nelng: params1.nelng.or(params2.nelng),
        swlat: params1.swlat.or(params2.swlat),
        swlng: params1.swlng.or(params2.swlng),
        list_id: params1.list_id.or(params2.list_id),
        not_in_project: params1.not_in_project.or(params2.not_in_project),
        not_matching_project_rules_for: params1
            .not_matching_project_rules_for
            .or(params2.not_matching_project_rules_for),
        q: params1.q.or(params2.q),
        search_on: params1.search_on.or(params2.search_on),
        quality_grade: params1.quality_grade.or(params2.quality_grade),
        updated_since: params1.updated_since.or(params2.updated_since),
        viewer_id: params1.viewer_id.or(params2.viewer_id),
        reviewed: params1.reviewed.or(params2.reviewed),
        locale: params1.locale.or(params2.locale),
        preferred_place_id: params1.preferred_place_id.or(params2.preferred_place_id),
        ttl: params1.ttl.or(params2.ttl),
        page: params1.page.or(params2.page),
        per_page: params1.per_page.or(params2.per_page),
        order: params1.order.or(params2.order),
        order_by: params1.order_by.or(params2.order_by),
        only_id: params1.only_id.or(params2.only_id),
        expected_nearby: params1.expected_nearby.or(params2.expected_nearby),
        hour: params1.hour.or(params2.hour),
        created_day: params1.created_day.or(params2.created_day),
        created_month: params1.created_month.or(params2.created_month),
        created_year: params1.created_year.or(params2.created_year),
        term_id_or_unknown: params1.term_id_or_unknown.or(params2.term_id_or_unknown),
        annotation_user_id: params1.annotation_user_id.or(params2.annotation_user_id),
        obscuration: params1.obscuration.or(params2.obscuration),
        observation_accuracy_experiment_id: params1
            .observation_accuracy_experiment_id
            .or(params2.observation_accuracy_experiment_id),
    }
}

pub async fn fetch_taxa(
    taxa_ids: Vec<i32>,
    api_token: &str,
) -> Result<
    inaturalist::models::TaxaShowResponse,
    inaturalist::apis::Error<inaturalist::apis::taxa_api::TaxaIdGetError>,
> {
    tracing::info!("Fetching taxa IDs = {:?}", taxa_ids);
    inaturalist_rate_limiter().until_ready().await;
    let taxa = inaturalist::apis::taxa_api::taxa_id_get(
        &get_inaturalist_request_config(api_token),
        inaturalist::apis::taxa_api::TaxaIdGetParams {
            id: taxa_ids.clone(),
            rank_level: None,
        },
    )
    .await?;
    tracing::info!("Fetched taxa IDs = {:?}", taxa_ids);
    Ok(taxa)
}

#[derive(Debug)]
pub enum FetchComputerVisionError {
    Unauthorized,
    Serde(serde_json::Error, String),
    Reqwest(reqwest::Error),
}

impl fmt::Display for FetchComputerVisionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchComputerVisionError::Unauthorized => {
                write!(f, "Unauthorized (likely expired token)")
            }
            FetchComputerVisionError::Serde(e, text) => {
                write!(f, "JSON deserialization error: {e} for response: {text}")
            }
            FetchComputerVisionError::Reqwest(e) => write!(f, "Request error: {e}"),
        }
    }
}

impl std::error::Error for FetchComputerVisionError {}

impl From<reqwest::Error> for FetchComputerVisionError {
    fn from(err: reqwest::Error) -> FetchComputerVisionError {
        FetchComputerVisionError::Reqwest(err)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ComputerVisionObservationScoreResponse {
    pub total_results: usize,
    pub page: usize,
    pub per_page: usize,
    pub results: Vec<ComputerVisionObservationScore>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ComputerVisionObservationScore {
    pub vision_score: f32,
    pub combined_score: f32,
    pub original_geo_score: f32,
    pub original_combined_score: f32,
    pub frequency_score: f32,
    pub taxon: inaturalist::models::ObservationTaxon,
}

pub async fn fetch_computer_vision_observation_scores(
    observation: &inaturalist::models::Observation,
    api_token: &str,
) -> Result<ComputerVisionObservationScoreResponse, FetchComputerVisionError> {
    let observation_id = observation.id.unwrap();
    tracing::info!("Fetch observation score (observation ID: {observation_id}");
    let url =
        format!("https://api.inaturalist.org/v1/computervision/score_observation/{observation_id}");
    inaturalist_rate_limiter().until_ready().await;
    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", api_token)
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let response_text = response.text().await?;
        tracing::error!(
            "Unauthorized fetching computer vision scores (observation ID = {}). Response: {:?}",
            observation.id.unwrap(),
            response_text
        );
        return Err(FetchComputerVisionError::Unauthorized);
    }

    let response = response.error_for_status()?;

    let response_text = response.text().await?;
    match serde_json::from_str(&response_text) {
        Ok(j) => Ok(j),
        Err(e) => {
            tracing::error!(
                "Could not deserialize computer vision observation scores (observation ID = {}). Response: {:?}",
                observation.id.unwrap(),
                response_text
            );
            Err(FetchComputerVisionError::Serde(e, response_text))
        }
    }
}

pub async fn identify(
    observation_id: i32,
    taxon_id: i32,
    api_token: &str,
) -> Result<(), impl std::error::Error> {
    inaturalist::apis::identifications_api::identifications_post(
        &get_inaturalist_request_config(api_token),
        inaturalist::apis::identifications_api::IdentificationsPostParams {
            body: Some(inaturalist::models::PostIdentification {
                identification: Some(Box::new(
                    inaturalist::models::PostIdentificationIdentification {
                        observation_id: Some(observation_id),
                        taxon_id: Some(taxon_id),
                        current: None,
                        body: None,
                    },
                )),
            }),
        },
    )
    .await
}
