use std::fs::File;

mod data;
mod trace;
mod tune;

pub fn main(filenames: &[String]) {
    let mut files = Vec::with_capacity(filenames.len());
    for filename in filenames {
        files.push(File::open(filename).expect("Unable to open policy data file"));
    }

    let dataset = data::load_dataset(files.as_slice());
    let params = &trace::zero_params();
    println!(
        "Uniform policy error: {}",
        tune::error_total(params, &dataset)
    );
    tune::optimize(params.clone(), &dataset);
}
