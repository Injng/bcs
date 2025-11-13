use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use natord::compare;
use rocket::serde::{Deserialize, Serialize, json};
use rocket::{State, get, launch, post, routes};
use serde::{Deserializer, Serializer};
use serde_json::from_reader;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, Schema, TextFieldIndexing, TextOptions, Value};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{DocAddress, Index, Score, Searcher, TantivyDocument, TantivyError, doc};

const LOAD: usize = 50;

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
    instructor: String,
    days: String,
    start: String,
    end: String,
    location: String,
    id: String,
    units: String,
    mode: String,
    #[serde(rename = "course-description")]
    course_description: String,
    #[serde(rename = "class-description")]
    class_description: String,
    capacity: u32,
    enrolled: u32,
    waitlist: u32,
    #[serde(rename = "waitlist-max")]
    waitlist_max: u32,
    requirements: Vec<Requirement>,
    seats: HashMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq)]
enum Requirement {
    AmericanCultures,
    AmericanHist,
    ArtsLiterature,
    BiologicalScience,
    EntryLevelWriting,
    HistoricalStudies,
    InternationalStudies,
    PhilosophyValues,
    PhysicalScience,
    ReadingCompA,
    ReadingCompB,
    SocialBehavioral,
    Other(String),
}

impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "the American Cultures requirement" => Ok(Requirement::AmericanCultures),
            "the American Hist & Institutions requirement" => Ok(Requirement::AmericanHist),
            "Arts & Literature, L&S Breadth" => Ok(Requirement::ArtsLiterature),
            "Biological Science, L&S Breadth" => Ok(Requirement::BiologicalScience),
            "the Entry Level Writing requirement" => Ok(Requirement::EntryLevelWriting),
            "Historical Studies, L&S Breadth" => Ok(Requirement::HistoricalStudies),
            "International Studies, L&S Breadth" => Ok(Requirement::InternationalStudies),
            "Philosophy & Values, L&S Breadth" => Ok(Requirement::PhilosophyValues),
            "Physical Science, L&S Breadth" => Ok(Requirement::PhysicalScience),
            "the Reading and Composition A requirement" => Ok(Requirement::ReadingCompA),
            "the Reading and Composition B requirement" => Ok(Requirement::ReadingCompB),
            "Social & Behavioral Sciences, L&S Breadth" => Ok(Requirement::SocialBehavioral),
            _ => Ok(Requirement::Other(s)),
        }
    }
}

impl Serialize for Requirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Requirement::AmericanCultures => "the American Cultures requirement",
            Requirement::AmericanHist => "the American Hist & Institutions requirement",
            Requirement::ArtsLiterature => "Arts & Literature, L&S Breadth",
            Requirement::BiologicalScience => "Biological Science, L&S Breadth",
            Requirement::EntryLevelWriting => "the Entry Level Writing requirement",
            Requirement::HistoricalStudies => "Historical Studies, L&S Breadth",
            Requirement::InternationalStudies => "International Studies, L&S Breadth",
            Requirement::PhilosophyValues => "Philosophy & Values, L&S Breadth",
            Requirement::PhysicalScience => "Physical Science, L&S Breadth",
            Requirement::ReadingCompA => "the Reading and Composition A requirement",
            Requirement::ReadingCompB => "the Reading and Composition B requirement",
            Requirement::SocialBehavioral => "Social & Behavioral Sciences, L&S Breadth",
            Requirement::Other(s) => s,
        };
        serializer.serialize_str(s)
    }
}

#[derive(Clone, Deserialize)]
struct Filter {
    requirements: Option<Vec<Requirement>>,
    requirements_or: bool,
}

impl Filter {
    fn filter_requirements(&self, course: &Class) -> bool {
        if let Some(r) = self.requirements.clone() {
            if self.requirements_or {
                for has in &(*course).requirements {
                    for needs in &r {
                        if *has == *needs {
                            return true;
                        }
                    }
                }
                return false;
            } else {
                for needs in &r {
                    let mut met = false;
                    for has in &(*course).requirements {
                        if *has == *needs {
                            met = true;
                            break;
                        }
                    }
                    if !met {
                        return false;
                    }
                }
                return true;
            }
        }
        return true;
    }

    fn filter(&self, course: &Class) -> bool {
        if !self.filter_requirements(course) {
            return false;
        }
        return true;
    }
}

#[derive(Clone, Deserialize)]
struct Query {
    keywords: String,
    offset: usize,
    filters: Filter,
}

#[get("/all")]
fn all(courses: &State<Vec<Class>>) -> json::Json<&Vec<Class>> {
    json::Json(&*courses)
}

fn get_results(
    mut query_struct: Query,
    courses: &State<Vec<Class>>,
    query_parser: &State<QueryParser>,
    searcher: &State<Searcher>,
    schema: &State<Schema>,
    classes: &mut Vec<Class>,
) -> Result<(), TantivyError> {
    // setup query and search for it in index
    let q = query_parser.parse_query(&query_struct.keywords)?;
    let results: Vec<(Score, DocAddress)> = searcher.search(
        &q,
        &TopDocs::with_limit(LOAD).and_offset(query_struct.offset),
    )?;
    if results.len() == 0 {
        return Ok(());
    }

    // map searched results to classes and ensure they meet filtered requirements
    let idx: Field = schema.get_field("idx").unwrap();
    for (_score, addr) in results {
        let retrieved = searcher.doc::<TantivyDocument>(addr)?;
        let i: u64 = retrieved.get_first(idx).and_then(|v| v.as_u64()).unwrap();
        let c: Class = (*courses)[i as usize].clone();
        if query_struct.filters.filter(&c) {
            classes.push(c.clone());
        }
    }

    // if not enough classes after filter, increment offset by 39 and try again
    if classes.len() < LOAD {
        query_struct.offset += LOAD;
        return get_results(
            query_struct,
            courses,
            query_parser,
            searcher,
            schema,
            classes,
        );
    }
    Ok(())
}

#[post("/search", format = "json", data = "<query>")]
fn search(
    query: json::Json<Query>,
    courses: &State<Vec<Class>>,
    query_parser: &State<QueryParser>,
    searcher: &State<Searcher>,
    schema: &State<Schema>,
) -> json::Json<Vec<Class>> {
    // get results from query
    let query_struct = query.into_inner();
    let mut classes: Vec<Class> = vec![];
    let results = get_results(
        query_struct.clone(),
        courses,
        query_parser,
        searcher,
        schema,
        &mut classes,
    );

    // if code starts with query, prioritize it, otherwise keep order
    let mut prefix_matches: Vec<Class> = classes
        .iter()
        .filter(|c| {
            c.code
                .to_lowercase()
                .starts_with(&query_struct.keywords.to_lowercase())
        })
        .cloned()
        .collect::<Vec<_>>();
    let other_matches = classes
        .iter()
        .filter(|c| {
            !c.code
                .to_lowercase()
                .starts_with(&query_struct.keywords.to_lowercase())
        })
        .cloned()
        .collect::<Vec<_>>();
    prefix_matches.extend(other_matches);

    match results {
        Ok(_) => json::Json(prefix_matches),
        Err(_) => json::Json(vec![]),
    }
}

#[launch]
fn launch() -> _ {
    // deserialize course json
    let file = File::open("classes.json").expect("Failed to open class file");
    let reader = BufReader::new(file);
    let mut courses: Vec<Class> = from_reader(reader).expect("Failed to parse JSON");
    courses.sort_by(|a, b| compare(&a.code, &b.code));

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
    let mut query_parser = QueryParser::for_index(
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
    query_parser.set_field_boost(code, 5.0);
    query_parser.set_field_boost(title, 3.0);
    query_parser.set_field_boost(subtitle, 1.5);
    query_parser.set_field_boost(special, 1.5);
    query_parser.set_field_boost(course_description, 0.5);
    query_parser.set_field_boost(class_description, 0.5);

    // launch routes
    rocket::build()
        .manage(courses)
        .manage(query_parser)
        .manage(searcher)
        .manage(schema)
        .mount("/classes", routes![all, search])
}
