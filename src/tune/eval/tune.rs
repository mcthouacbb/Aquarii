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

pub fn compute_single_grad(params: &Vec<f32>, grads: &mut Vec<f32>, pos: &Position, scale: f32) {
    let eval = eval_eval_wdl(params, pos, scale);
    let grad_base = (eval - pos.wdl) * eval * (1.0 - eval);

    for coeff in &pos.coeffs {
        grads[coeff.index as usize] += grad_base * coeff.value;
    }
}

pub fn compute_grads(params: &Vec<f32>, grads: &mut Vec<f32>, positions: &[Position], scale: f32) {
    for pos in positions {
        compute_single_grad(params, grads, pos, scale);
    }
    for grad in grads {
        *grad /= positions.len() as f32;
    }
}

pub fn optimize(mut params: Vec<f32>, dataset: &Dataset) {
    const BETA1: f32 = 0.9;
    const BETA2: f32 = 0.999;
    const EPSILON: f32 = 1e-8;
    const LR: f32 = 0.05;
    const BATCH_SIZE: u32 = 16384;
    const SUPERBATCH_SIZE: u32 = 6104;

    // change this soon
    let scale = 400.0;

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
            scale,
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
                error_total(&params, dataset, scale),
                num_batches as f32 / start_time.elapsed().as_secs_f32()
            );
        }

        if num_batches % SUPERBATCH_SIZE == 0 {
            println!(
                "SuperBatch {} error {}",
                num_batches / SUPERBATCH_SIZE,
                error_total(&params, dataset, scale)
            );
            println!("{}", trace::EvalFeature::format_all_features(&params));
        }
    }
}
