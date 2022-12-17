//! Manage City Rating Scorecards.
//!
//! This module contains the different structures used to compose a ScoreCard as
//! defined in the City Ratings.
//!
//! This module contains Python wrappers, generated by
//! [Py03](https://github.com/PyO3/PyO3). Some of these wrappers are just
//! aliases to other functions, but with a definition that makes them Python
//! compatible. For example, Python does not understand generics, or cannot use
//! slices `&[T]`. Refer to the
//! [Mapping of Rust types to Python types](https://pyo3.rs/v0.16.3/conversions/tables.html)
//! chapter of the Py03 book for more details.
use crate::{Dataset, Error, PFB_S3_PUBLIC_DOCUMENTS, PFB_S3_STORAGE_BASE_URL};
use csv::Reader;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;

/// Represent a PeopleForBikes city.
#[pyclass]
#[derive(Debug, Deserialize, Clone)]
pub struct City {
    /// City name.
    #[pyo3(get, set)]
    #[serde(rename = "City")]
    pub name: String,
    /// Country where the city is located.
    #[pyo3(get, set)]
    #[serde(rename = "Country")]
    pub country: String,
    /// State where the city is located.
    #[pyo3(get, set)]
    #[serde(rename = "State")]
    pub state: String,
    /// City's unique identifier.
    ///
    /// It is generated by a specific Bicyle Network Analysis (BNA) run and
    /// should be assimilated to a version number (each run will generate a
    /// new identifier).
    #[pyo3(get, set)]
    pub uuid: String,
    /// City's population.
    #[pyo3(get, set)]
    pub population: u32,
    /// City rating.
    #[pyo3(get, set)]
    #[serde(rename = "city_ratings_total")]
    pub ratings: f64,
    /// Rounded city rating.
    #[pyo3(get, set)]
    #[serde(rename = "city_ratings_rounded")]
    pub ratings_rounded: u8,
}

/// Define Python compatible methods.
#[pymethods]
impl City {
    /// Create a new City.
    ///
    /// If the `state` is not specified (a lot of countries do not have states),
    /// the name of the country is used instead.
    #[new]
    pub fn new(
        name: &str,
        country: &str,
        state: Option<&str>,
        uuid: &str,
        population: u32,
        ratings: f64,
        ratings_rounded: u8,
    ) -> Self {
        City {
            name: name.into(),
            country: country.into(),
            state: if let Some(s) = state {
                s.into()
            } else {
                country.into()
            },
            uuid: uuid.into(),
            population,
            ratings,
            ratings_rounded,
        }
    }
}

impl City {
    /// Return the full name of the city.
    ///
    /// The full name has the following format: `{COUNTRY}-{STATE}-{CITY_NAME}`.
    pub fn full_name(&self) -> String {
        format!("{}-{}-{}", self.country, self.state, self.name)
    }

    /// Return the URL of the specified dataset.
    pub fn url(&self, dataset: &Dataset) -> Result<Url, Error> {
        let mut dataset_url: String = String::new();
        if *dataset == Dataset::DataDictionary {
            dataset_url.push_str(PFB_S3_PUBLIC_DOCUMENTS);
        } else {
            dataset_url.push_str(PFB_S3_STORAGE_BASE_URL);
            dataset_url.push('/');
            dataset_url.push_str(&self.uuid);
        }
        dataset_url.push('/');
        dataset_url.push_str(&dataset.to_string());
        dataset_url.push('.');
        dataset_url.push_str(&dataset.extension());
        Ok(Url::parse(&dataset_url)?)
    }

    /// Read a CSV file and populate a Vector of Cities.
    pub fn from_csv<P>(path: P) -> Result<Vec<City>, Error>
    where
        P: AsRef<Path>,
    {
        let mut csv_reader = Reader::from_path(path)?;
        let mut cities: Vec<City> = vec![];
        for record in csv_reader.deserialize() {
            cities.push(record?);
        }

        Ok(cities)
    }
}

/// Represent the results from the community survey.
#[pyclass]
#[derive(Debug, Deserialize, Clone)]
pub struct CommunitySurvey {
    /// Perception of the quality of the bicycle network in the city.
    #[pyo3(get, set)]
    #[serde(rename = "Community Survey - Network")]
    pub network: f64,
    /// Perceptions of acceleration and awareness of bike events and facilities in an area.
    #[pyo3(get, set)]
    #[serde(rename = "Community Survey - Awareness")]
    pub awareness: f64,
    /// Perceptions of safety riding a bike .
    #[pyo3(get, set)]
    #[serde(rename = "Community Survey - Safety")]
    pub safety: f64,
    /// Measure how often respondents engage in different types of riding.
    #[pyo3(get, set)]
    #[serde(rename = "Community Survey - Ridership")]
    pub ridership: f64,
    /// Overall community survey score.
    #[pyo3(get, set)]
    #[serde(rename = "Community Score - Total")]
    pub total: f64,
    /// Overall community survey rounded score.
    #[pyo3(get, set)]
    #[serde(rename = "Community Score - Total, Rounded")]
    pub total_rounded: u32,
    /// Number of responses to the survey.
    #[pyo3(get, set)]
    #[serde(rename = "Community Survey - Responses")]
    pub responses: u32,
}

/// Represent the results from the BNA.
#[derive(Debug, Deserialize, Clone)]
#[pyclass]
pub struct BNA {
    /// How well people can reach other people by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - neighborhoods")]
    pub neighborhoods: f64,
    /// How well people can reach employment and educational opportunities by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - opportunity")]
    pub opportunity: f64,
    /// How well people can reach Core Services by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - essential_services")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub essential_services: Option<f64>,
    /// How well people can reach retail shopping opportunities by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - retail")]
    pub retail: f64,
    /// How well people can reach recreation opportunities by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - recreation")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub recreation: Option<f64>,
    /// How well people can reach major transit hubs by bike.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - transit")]
    pub transit: f64,
    /// How well the bike network gets people to the places they want to go.
    #[pyo3(get, set)]
    #[serde(rename = "BNA - overall_score")]
    pub overall_score: f64,
}

/// Represent a city bike infrastructure.
#[pyclass]
#[derive(Debug, Deserialize, Clone)]
pub struct Infrastructure {
    /// Miles of low stress infrstructure.
    #[pyo3(get, set)]
    #[serde(rename = "total_low_stress_miles")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub low_stress_miles: Option<f64>,
    /// Miles of high stress infrastructure.
    #[pyo3(get, set)]
    #[serde(rename = "total_high_stress_miles")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub high_stress_miles: Option<f64>,
}

/// Represent a city scorecard.
#[pyclass]
#[derive(Debug, Deserialize, Clone)]
pub struct ScoreCard {
    /// City details.
    #[pyo3(get, set)]
    #[serde(flatten)]
    pub city: City,
    /// Community survey results.
    #[pyo3(get, set)]
    #[serde(flatten)]
    pub community_survey: CommunitySurvey,
    /// BNA results.
    #[pyo3(get, set)]
    #[serde(flatten)]
    pub bna: BNA,
    /// Infrastructure details.
    #[pyo3(get, set)]
    #[serde(flatten)]
    pub infrastructure: Infrastructure,
}

impl ScoreCard {
    /// Read a CSV file and populate a Vector of ScoreCards.
    pub fn from_csv<P>(path: P) -> Result<Vec<ScoreCard>, Error>
    where
        P: AsRef<Path>,
    {
        let mut csv_reader = Reader::from_path(path)?;
        let mut scorecards: Vec<ScoreCard> = vec![];
        for record in csv_reader.deserialize() {
            scorecards.push(record?);
        }

        Ok(scorecards)
    }
}

/// Define Python compatible methods.
#[pymethods]
impl ScoreCard {
    /// Python wrapper for the [`ScoreCard::from_csv`] method.
    #[staticmethod]
    pub fn load_csv(path: &str) -> PyResult<Vec<ScoreCard>> {
        Ok(ScoreCard::from_csv(path)?)
    }
}

/// Represent a ScoreCard to be passed to `svggloo`.
///
/// The fields must match all the fields from ScoreCard, and be represented by
/// their short forms.
#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct ShortScoreCard {
    /// City
    #[pyo3(get, set)]
    pub ci: String,
    #[pyo3(get, set)]
    pub co: String,
    #[pyo3(get, set)]
    pub st: String,
    #[pyo3(get, set)]
    pub uuid: String,
    #[pyo3(get, set)]
    pub po: u32,
    #[pyo3(get, set)]
    pub ra: f64,
    #[pyo3(get, set)]
    pub rasc: u8,

    // Community Survey
    #[pyo3(get, set)]
    pub nw: u8,
    #[pyo3(get, set)]
    pub aw: u8,
    #[pyo3(get, set)]
    pub sf: u8,
    #[pyo3(get, set)]
    pub rs: u8,
    #[pyo3(get, set)]
    pub total: u8,
    #[pyo3(get, set)]
    pub cssc: u8,
    #[pyo3(get, set)]
    pub responses: u32,

    // BNA
    #[pyo3(get, set)]
    pub nh: u8,
    #[pyo3(get, set)]
    pub op: u8,
    #[pyo3(get, set)]
    pub es: u8,
    #[pyo3(get, set)]
    pub ret: u8,
    #[pyo3(get, set)]
    pub rec: u8,
    #[pyo3(get, set)]
    pub tr: u8,
    #[pyo3(get, set)]
    pub bnasc: u8,

    // Infrastructure
    #[pyo3(get, set)]
    pub lsm: u32,
    #[pyo3(get, set)]
    pub hsm: u32,
}

impl From<&ScoreCard> for ShortScoreCard {
    fn from(sc: &ScoreCard) -> Self {
        ShortScoreCard {
            ci: sc.city.name.clone(),
            co: sc.city.country.clone(),
            st: sc.city.state.clone(),
            uuid: sc.city.uuid.clone(),
            po: sc.city.population,
            ra: sc.city.ratings,
            rasc: sc.city.ratings_rounded,
            nw: sc.community_survey.network.round() as u8,
            aw: sc.community_survey.awareness.round() as u8,
            sf: sc.community_survey.safety.round() as u8,
            rs: sc.community_survey.ridership.round() as u8,
            total: sc.community_survey.total.round() as u8,
            cssc: sc.community_survey.total_rounded as u8,
            responses: sc.community_survey.responses,
            nh: sc.bna.neighborhoods.round() as u8,
            op: sc.bna.opportunity.round() as u8,
            es: sc.bna.essential_services.unwrap_or_default().round() as u8,
            ret: sc.bna.retail.round() as u8,
            rec: sc.bna.recreation.unwrap_or_default().round() as u8,
            tr: sc.bna.transit.round() as u8,
            bnasc: sc.bna.overall_score.round() as u8,
            lsm: sc
                .infrastructure
                .low_stress_miles
                .unwrap_or_default()
                .round() as u32,
            hsm: sc
                .infrastructure
                .high_stress_miles
                .unwrap_or_default()
                .round() as u32,
        }
    }
}

impl ShortScoreCard {
    // Saves a slice of ShortScoreCards to a CSV file.
    pub fn to_csv<P>(path: P, entries: &[ShortScoreCard]) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let mut w = csv::Writer::from_path(path)?;
        for entry in entries {
            w.serialize(entry)?;
        }
        Ok(w.flush()?)
    }
}

/// Define Python compatible methods.
#[pymethods]
impl ShortScoreCard {
    /// Python wrapper for the [`ShortScoreCard::to_csv`] method.
    #[staticmethod]
    pub fn save_csv(path: &str, entries: Vec<ShortScoreCard>) -> PyResult<()> {
        Ok(ShortScoreCard::to_csv(path, &entries)?)
    }
}
