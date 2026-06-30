//! Generate the checked-in Project JSON Schema. Run:
//!   cargo run -p fpa-domain --example gen_schema --features schema > crates/fpa-domain/schema/project.schema.json
fn main() {
    let schema = schemars::schema_for!(fpa_domain::Project);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
