use futures::{FutureExt, StreamExt};
use geo_ext::Halve;
use std::{num, pin::Pin, sync};

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

const INATURALIST_RATE_LIMIT_AMOUNT: governor::Quota =
    governor::Quota::per_minute(unsafe { num::NonZeroU32::new_unchecked(30) });

lazy_static::lazy_static! {
    static ref INATURALIST_REQUEST_CONFIG: inaturalist::apis::configuration::Configuration =
        inaturalist::apis::configuration::Configuration {
            base_path: String::from("https://api.inaturalist.org/v1"),
            ..Default::default()
        };

    pub static ref INATURALIST_RATE_LIMITER: governor::RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    > =
        governor::RateLimiter::direct(INATURALIST_RATE_LIMIT_AMOUNT);
}

const AUTHORIZATION: &str = "eyJhbGciOiJIUzUxMiJ9.eyJ1c2VyX2lkIjozMTkxNDIyLCJvYXV0aF9hcHBsaWNhdGlvbl9pZCI6ODEzLCJleHAiOjE2Nzc2MzIxODV9.0ymO3vIRqUJ1DCFT8HWYrvkIINJxA8lkat5PTNNFcBZLGfgHIExoYiee7iHOKy9WtoruSGN2BX0PnN6M3kNFyA";

#[derive(Copy, Clone)]
pub struct SubdividedRect(pub crate::Rect);

/// iNaturalist will not let us page past 10,000 results.
const MAX_RESULTS: i32 = 10_000;

const MAX_RESULTS_PER_PAGE: u32 = 200;

type SubdivideRectReturn = Pin<
    Box<
        dyn futures::Stream<
                Item = Result<
                    SubdividedRect,
                    inaturalist::apis::Error<
                        inaturalist::apis::observations_api::ObservationsGetError,
                    >,
                >,
            > + Send,
    >,
>;

#[async_recursion::async_recursion]
pub async fn subdivide_rect(rect: Rect) -> SubdivideRectReturn {
    let page = 1;
    let per_page = 1;
    INATURALIST_RATE_LIMITER.until_ready().await;

    let response = match inaturalist::apis::observations_api::observations_get(
        &INATURALIST_REQUEST_CONFIG,
        build_params(rect, page, per_page),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return Box::pin(futures::future::err(e).into_stream()) as SubdivideRectReturn,
    };

    if response.total_results.unwrap() < MAX_RESULTS {
        tracing::info!("Rect is sufficient");
        return Box::pin(futures::future::ok(SubdividedRect(rect)).into_stream())
            as SubdivideRectReturn;
    }

    tracing::info!(
        "Splitting rect (total_results: {})",
        response.total_results.unwrap()
    );
    let (rect1, rect2) = rect.halve();

    Box::pin(futures::stream::select(
        subdivide_rect(rect1).await,
        subdivide_rect(rect2).await,
    ))
}

pub async fn fetch(
    rect: Rect,
    tx: tokio::sync::mpsc::UnboundedSender<inaturalist::models::Observation>,
    soft_limit: &sync::atomic::AtomicI32,
    request: inaturalist::apis::observations_api::ObservationsGetParams,
) -> Result<(), inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>>
{
    let per_page = MAX_RESULTS_PER_PAGE;

    for page in 1.. {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            tracing::info!("Hit soft limit.");
            break;
        }

        tracing::info!("Fetching observations...");
        INATURALIST_RATE_LIMITER.until_ready().await;
        let response = inaturalist::apis::observations_api::observations_get(
            &INATURALIST_REQUEST_CONFIG,
            merge_params(request.clone(), build_params(rect, page, per_page)),
        )
        .await?;

        soft_limit.fetch_sub(
            response.results.len() as i32,
            sync::atomic::Ordering::Relaxed,
        );
        for result in response.results {
            tx.send(result).unwrap();
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
        order: Some("desc".to_string()),
        order_by: Some("observed_on".to_string()),
        per_page: Some(per_page.to_string()),
        page: Some(page.to_string()),
        ..Default::default()
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
    }
}

pub async fn fetch_taxa(
    taxa_ids: Vec<i32>,
) -> Result<
    inaturalist::models::TaxaShowResponse,
    inaturalist::apis::Error<inaturalist::apis::taxa_api::TaxaIdGetError>,
> {
    INATURALIST_RATE_LIMITER.until_ready().await;
    let taxa = inaturalist::apis::taxa_api::taxa_id_get(
        &INATURALIST_REQUEST_CONFIG,
        inaturalist::apis::taxa_api::TaxaIdGetParams { id: taxa_ids },
    )
    .await?;
    Ok(taxa)
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
) -> ComputerVisionObservationScoreResponse {
    let observation_id = observation.id.unwrap();
    tracing::info!("Fetch observation score (observation ID: {observation_id}");
    let url =
        format!("https://api.inaturalist.org/v1/computervision/score_observation/{observation_id}");
    INATURALIST_RATE_LIMITER.until_ready().await;
    reqwest::Client::new()
        .get(url)
        .header("Authorization", AUTHORIZATION)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
