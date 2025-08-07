use std::fs::File;

use crate::chess::Board;

mod data;
mod trace;
mod tune;

pub fn main(filenames: &[String]) {
    let mut files = Vec::with_capacity(filenames.len());
    for filename in filenames {
        files.push(File::open(filename).expect("Unable to open value data file"));
    }

    let dataset = data::load_dataset(files.as_slice());
    let params = &trace::zero_params();
    println!("{}", trace::EvalFeature::format_all_features(params));
    println!(
        "Draw eval error: {}",
        tune::error_total(params, &dataset, 400.0)
    );
    tune::optimize(params.clone(), &dataset);
}
