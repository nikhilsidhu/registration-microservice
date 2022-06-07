fn main() {
  // rebuilds crates if templates have changed
  askama::rerun_if_templates_changed();
}