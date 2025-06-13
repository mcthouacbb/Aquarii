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

pub fn compute_grads(params: &Vec<f32>, grads: &mut Vec<f32>, dataset: &Dataset) {
    for pos in &dataset.positions {
        compute_single_grad(params, grads, pos);
    }
    for grad in grads {
        *grad /= dataset.positions.len() as f32;
    }
}

/*pub fn compute_grads_slow(params: &mut Vec<f32>, grads: &mut Vec<f32>, dataset: &Dataset) {
    const EPSILON: f32 = 0.00001;
    for i in 0..params.len() {
        let old_error = error_total(params, dataset);
        let old = params[i];

        params[i] += EPSILON;
        let new_error = error_total(params, dataset);
        params[i] = old;

        grads[i] = (new_error - old_error) / EPSILON;
    }
}

pub fn compare_slow_fast(params: &Vec<f32>, dataset: &Dataset) {
    let mut grads_slow = params.clone();
    grads_slow.fill(0.0);
    let mut grads_fast = params.clone();
    grads_fast.fill(0.0);
    compute_grads(&mut params.clone(), &mut grads_fast, dataset);
    compute_grads_slow(&mut params.clone(), &mut grads_slow, dataset);

    for i in 0..params.len() {
        if (grads_slow[i] - grads_fast[i]).abs() > 0.001 {
            println!("{} slow {} fast {}", i, grads_slow[i], grads_fast[i])
        }
    }
}*/

pub fn optimize(mut params: Vec<f32>, dataset: &Dataset) {
    const BETA1: f32 = 0.9;
    const BETA2: f32 = 0.999;
    const EPSILON: f32 = 1e-8;
    const LR: f32 = 0.1;

    let mut grads = params.clone();
    let mut velocity = params.clone();
    let mut momentum = params.clone();
    grads.fill(0.0);
    velocity.fill(0.0);
    momentum.fill(0.0);

    for i in 1..=100000 {
        compute_grads(&params, &mut grads, dataset);
        // compare_slow_fast(&params, dataset);
        for j in 0..params.len() {
            momentum[j] = BETA1 * momentum[j] + (1.0 - BETA1) * grads[j];
            velocity[j] = BETA2 * velocity[j] + (1.0 - BETA2) * grads[j] * grads[j];
            params[j] -= LR * momentum[j] / (velocity[j].sqrt() + EPSILON);
        }

        println!("Epoch {} error {}", i, error_total(&params, dataset));
        if i % 25 == 0 {
            println!("{}", trace::PolicyFeature::format_all_features(&params));
        }
    }
}
