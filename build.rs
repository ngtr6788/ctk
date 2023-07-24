use ctk_common::cli_parser::ColdTurkey;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};

fn main() {
  let mut cold_turkey = ColdTurkey::command_for_update();
  let outdir = env!("CARGO_MANIFEST_DIR");
  generate_to(Bash, &mut cold_turkey, "ctk", outdir); 
}
