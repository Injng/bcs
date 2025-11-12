use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use rocket::serde::{Deserialize, Serialize, json};
use rocket::{State, get, launch, post, routes};
use serde_json::from_reader;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, Schema, TextFieldIndexing, TextOptions, Value};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer, TextAnalyzerBuilder};
use tantivy::{DocAddress, Index, Score, Searcher, TantivyDocument, TantivyError, doc};

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Deserialize)]
struct Query {
    keywords: String,
}

#[get("/all")]
fn all(courses: &State<Vec<Class>>) -> json::Json<&Vec<Class>> {
    json::Json(&*courses)
}

fn get_results(
    query: json::Json<Query>,
    courses: &State<Vec<Class>>,
    query_parser: &State<QueryParser>,
    searcher: &State<Searcher>,
    schema: &State<Schema>,
) -> Result<Vec<Class>, TantivyError> {
    let q = query_parser.parse_query(&query.into_inner().keywords)?;
    let results: Vec<(Score, DocAddress)> = searcher.search(&q, &TopDocs::with_limit(10))?;
    let mut classes: Vec<Class> = vec![];
    let idx: Field = schema.get_field("idx").unwrap();
    for (_score, addr) in results {
        let retrieved = searcher.doc::<TantivyDocument>(addr)?;
        let i: u64 = retrieved.get_first(idx).and_then(|v| v.as_u64()).unwrap();
        classes.push((*courses)[i as usize].clone());
    }
    Ok(classes)
}

#[post("/search", format = "json", data = "<query>")]
fn search(
    query: json::Json<Query>,
    courses: &State<Vec<Class>>,
    query_parser: &State<QueryParser>,
    searcher: &State<Searcher>,
    schema: &State<Schema>,
) -> json::Json<Vec<Class>> {
    let results = get_results(query, courses, query_parser, searcher, schema);
    match results {
        Ok(classes) => json::Json(classes),
        Err(_) => json::Json(vec![]),
    }
}

#[launch]
fn launch() -> _ {
    // deserialize course json
    let file = File::open("classes.json").expect("Failed to open class file");
    let reader = BufReader::new(file);
    let courses: Vec<Class> = from_reader(reader).expect("Failed to parse JSON");

    // schema for searching classes
    let o = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("course_search")
            .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
    );
    let mut schema_builder = Schema::builder();
    let idx = schema_builder.add_u64_field("idx", STORED);
    let code = schema_builder.add_text_field("code", o.clone());
    let title = schema_builder.add_text_field("title", o.clone());
    let subtitle = schema_builder.add_text_field("subtitle", o.clone());
    let special = schema_builder.add_text_field("special", o.clone());
    let course_description = schema_builder.add_text_field("course_description", o.clone());
    let class_description = schema_builder.add_text_field("class_description", o.clone());
    let schema = schema_builder.build();

    // index the courses
    let index = Index::create_in_ram(schema.clone());
    let analyzer = TextAnalyzer::builder(NgramTokenizer::new(3, 6, false).unwrap())
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("course_search", analyzer);
    let mut writer = index.writer(50_000_000).unwrap();
    for i in 0..courses.len() {
        let c = courses[i].clone();
        writer
            .add_document(doc!(
                idx => i as u64,
                code => c.code,
                title => c.title,
                subtitle => c.subtitle,
                special => c.special,
                course_description => c.course_description,
                class_description => c.class_description
            ))
            .unwrap();
    }
    writer.commit().unwrap();

    // setup searching
    let searcher: Searcher = index.reader().unwrap().searcher();
    let query_parser = QueryParser::for_index(
        &index,
        vec![
            code,
            title,
            subtitle,
            special,
            course_description,
            class_description,
        ],
    );

    // launch routes
    rocket::build()
        .manage(courses)
        .manage(query_parser)
        .manage(searcher)
        .manage(schema)
        .mount("/classes", routes![all, search])
}
