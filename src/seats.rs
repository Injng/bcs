use crate::Reservation;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

fn add_colleges(r: &Reservation) {}

fn add_majors(r: &Reservation) {}

fn add_minors(r: &Reservation) {}

fn add_terms(r: &Reservation) {}

fn add_transfer(r: &Reservation) {}

fn add_levels(r: &Reservation) {}

/*
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Reservation>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: HashMap<String, u32> = HashMap::deserialize(deserializer)?;
    let mut result: HashMap<Reservation, u32>;
}
    */
