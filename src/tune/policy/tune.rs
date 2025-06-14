use arrayvec::ArrayVec;

use crate::tune::policy::{
    data::{Dataset, Position},
    trace,
};

fn eval_policy(params: &Vec<f32>, pos: &Position) -> ArrayVec<f32, 256> {
    let mut result = ArrayVec::new();

    for i in 0..pos.movecount {
        result.push(0.0);
    }

    for coeff in &pos.coeffs {
        result[coeff.mv_idx as usize] += params[coeff.index as usize] * coeff.value;
    }

    let mut max = -100000.0f32;
    for elem in &result {
        max = max.max(*elem);
    }

    let mut sum = 0.0f32;
    for elem in result.iter_mut() {
        *elem = (*elem - max).exp();
        sum += *elem;
    }

    for elem in result.iter_mut() {
        *elem /= sum;
    }

    result
}

fn error_single(params: &Vec<f32>, pos: &Position) -> f32 {
    let policy = eval_policy(params, pos);
    let mut loss = 0.0;
    for i in 0..pos.movecount {
        loss -= pos.visit_dist[i as usize] * policy[i as usize].ln();
    }
    loss
}

pub fn error_total(params: &Vec<f32>, dataset: &Dataset) -> f32 {
    let mut total = 0.0;
    for pos in &dataset.positions {
        total += error_single(params, pos);
    }
    total / dataset.positions.len() as f32
}

pub fn compute_single_grad(params: &Vec<f32>, grads: &mut Vec<f32>, pos: &Position) {
    let policy = eval_policy(params, pos);

    for coeff in &pos.coeffs {
        let predicted = policy[coeff.mv_idx as usize];
        let actual = pos.visit_dist[coeff.mv_idx as usize];
        let grad_contrib = coeff.value * (predicted - actual);
        // println!("idx: {} value: {} grad: {}", coeff.index, coeff.value, grad_contrib);
        grads[coeff.index as usize] += grad_contrib;
    }
}

pub fn compute_grads(params: &Vec<f32>, grads: &mut Vec<f32>, positions: &[Position]) {
    for pos in positions {
        compute_single_grad(params, grads, pos);
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

    let mut grads = params.clone();
    let mut velocity = params.clone();
    let mut momentum = params.clone();
    grads.fill(0.0);
    velocity.fill(0.0);
    momentum.fill(0.0);

    let mut batch_idx = 0;
    let mut num_batches = 0;
    loop {
        let begin_idx = batch_idx * BATCH_SIZE as usize;
        let end_idx = (batch_idx + 1) * BATCH_SIZE as usize;
        if end_idx > dataset.positions.len() {
            batch_idx = 0;
            continue;
        }
        compute_grads(&params, &mut grads, &dataset.positions[begin_idx..end_idx]);
        // compare_slow_fast(&params, dataset);
        for i in 0..params.len() {
            momentum[i] = BETA1 * momentum[i] + (1.0 - BETA1) * grads[i];
            velocity[i] = BETA2 * velocity[i] + (1.0 - BETA2) * grads[i] * grads[i];
            params[i] -= LR * momentum[i] / (velocity[i].sqrt() + EPSILON);
        }
        batch_idx += 1;
        num_batches += 1;

        if num_batches % 100 == 0 {
            println!(
                "Batch {} error {}",
                num_batches,
                error_total(&params, dataset)
            );
        }

        if num_batches % SUPERBATCH_SIZE == 0 {
            println!(
                "SuperBatch {} error {}",
                num_batches / SUPERBATCH_SIZE,
                error_total(&params, dataset)
            );
            println!("{}", trace::PolicyFeature::format_all_features(&params));
        }
    }
}
