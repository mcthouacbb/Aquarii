use std::time::Instant;

use crate::tune::eval::{
    data::{Dataset, Position},
    trace,
};

fn eval_eval_cp(params: &Vec<f32>, pos: &Position) -> f32 {
    let mut result = 0.0;
    for coeff in &pos.coeffs {
        result += params[coeff.index as usize] * coeff.value;
    }

    return result;
}

fn eval_eval_wdl(params: &Vec<f32>, pos: &Position, scale: f32) -> f32 {
    return 1.0 / (1.0 + (-eval_eval_cp(params, pos) / scale).exp());
}

fn error_single(params: &Vec<f32>, pos: &Position, scale: f32) -> f32 {
    let eval = eval_eval_wdl(params, pos, scale);
    let wdl = pos.wdl;
    return (eval - wdl) * (eval - wdl);
}

pub fn error_total(params: &Vec<f32>, dataset: &Dataset, scale: f32) -> f32 {
    let mut total = 0.0;
    for pos in &dataset.positions {
        total += error_single(params, pos, scale);
    }
    total / dataset.positions.len() as f32
}

fn material_error(dataset: &Dataset, k: f32) -> f32 {
    let mut total = 0.0;
    for pos in &dataset.positions {
        let target = pos.score;
        let material = 1.0 / (1.0 + (-pos.default_material as f32 * k).exp());
        total += (material - target) * (material - target);
    }
    total / dataset.positions.len() as f32
}

pub fn compute_eval_scale(dataset: &Dataset) -> f32 {
    let mut best_k = 1f32 / 200f32;
    let mut best_error = 100f32;
    let mut start = 0f32;
    let mut end = 1f32 / 50f32;
    let mut step = 1f32 / 400f32;

    for _ in 0..7 {
        println!("{} {} {}", start, end, step);
        let mut curr_k = start + step;
        while curr_k < end + step {
            let error = material_error(dataset, curr_k);
            if error < best_error {
                best_error = error;
                best_k = curr_k;
                println!("New best: {}, error: {}", best_k, best_error);
            }
            curr_k += step;
        }

        start = best_k - step;
        end = best_k + step;
        step *= 0.1;
    }

    1.0 / best_k
}

pub fn compute_single_grad(params: &Vec<f32>, grads: &mut Vec<f32>, pos: &Position, scale: f32) {
    let eval = eval_eval_wdl(params, pos, scale);
    let target = pos.score;
    let grad_base = (eval - target) * eval * (1.0 - eval);

    for coeff in &pos.coeffs {
        grads[coeff.index as usize] += grad_base * coeff.value;
    }
}

pub fn compute_grads(params: &Vec<f32>, grads: &mut Vec<f32>, positions: &[Position], scale: f32) {
    for pos in positions {
        compute_single_grad(params, grads, pos, scale);
    }
    for grad in grads {
        *grad /= scale * positions.len() as f32;
    }
}

pub fn optimize(mut params: Vec<f32>, dataset: &Dataset) {
    const BETA1: f32 = 0.9;
    const BETA2: f32 = 0.999;
    const EPSILON: f32 = 1e-8;
    const LR: f32 = 1.0;
    // const LR: f32 = 0.05;
    const BATCH_SIZE: u32 = 65536;
    // let BATCH_SIZE: u32 = dataset.positions.len() as u32;
    const SUPERBATCH_SIZE: u32 = 1000;

    let eval_scale = compute_eval_scale(dataset);
    println!("Eval scale: {}", eval_scale);

    let mut grads = params.clone();
    let mut velocity = params.clone();
    let mut momentum = params.clone();
    grads.fill(0.0);
    velocity.fill(0.0);
    momentum.fill(0.0);

    let mut batch_idx = 0;
    let mut num_batches = 0;
    let start_time = Instant::now();
    loop {
        let begin_idx = batch_idx * BATCH_SIZE as usize;
        let end_idx = (batch_idx + 1) * BATCH_SIZE as usize;
        if end_idx > dataset.positions.len() {
            batch_idx = 0;
            continue;
        }
        grads.fill(0.0);
        compute_grads(
            &params,
            &mut grads,
            &dataset.positions[begin_idx..end_idx],
            eval_scale,
        );
        // compare_slow_fast(&params, dataset);
        // println!("{:?}", &grads[0..5]);
        for i in 0..params.len() {
            momentum[i] = BETA1 * momentum[i] + (1.0 - BETA1) * grads[i];
            velocity[i] = BETA2 * velocity[i] + (1.0 - BETA2) * grads[i] * grads[i];
            params[i] -= LR * momentum[i] / (velocity[i].sqrt() + EPSILON);
        }
        batch_idx += 1;
        num_batches += 1;

        if num_batches % 100 == 0 {
            println!(
                "Batch {} error {}, batches/s: {}",
                num_batches,
                error_total(&params, dataset, eval_scale),
                num_batches as f32 / start_time.elapsed().as_secs_f32()
            );
        }

        if num_batches % SUPERBATCH_SIZE == 0 {
            println!(
                "SuperBatch {} error {}",
                num_batches / SUPERBATCH_SIZE,
                error_total(&params, dataset, eval_scale)
            );
            println!("{}", trace::EvalFeature::format_all_features(&params));
        }
    }
}
