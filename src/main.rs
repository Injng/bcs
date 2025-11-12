use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use rocket::serde::{Deserialize, Serialize, json};
use rocket::{State, get, launch, routes};
use serde_json::from_reader;

#[derive(Serialize, Deserialize)]
struct Class {
    code: String,
    #[serde(rename = "class-type")]
    class_type: String,
    count: String,
    title: String,
    subtitle: String,
    special: String,
    link: String,
    location: String,
    id: String,
    units: String,
    #[serde(rename = "course-description")]
    course_description: String,
    #[serde(rename = "class-description")]
    class_description: String,
    capacity: u32,
    enrolled: u32,
    seats: HashMap<String, u32>,
}

#[get("/all")]
fn all(courses: &State<Vec<Class>>) -> json::Json<&Vec<Class>> {
    json::Json(&*courses)
}

#[launch]
fn launch() -> _ {
    // deserialize course json
    let file = File::open("classes.json").expect("Failed to open class file");
    let reader = BufReader::new(file);
    let courses: Vec<Class> = from_reader(reader).expect("Failed to parse JSON");

    // launch routes
    rocket::build()
        .manage(courses)
        .mount("/classes", routes![all])
}
