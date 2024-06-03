use std::time::{Duration, Instant};

use ark_bn254::{Bn254, Fr as F, G1Projective as G1, G2Projective as G2};
use ark_ec::short_weierstrass::Projective;
use ark_ff::Field;
use ark_poly::{polynomial::Polynomial, EvaluationDomain, GeneralEvaluationDomain};
use ark_poly::{univariate::DensePolynomial, Evaluations};
use ark_std::Zero;
use ark_std::{test_rng, UniformRand};
use kzg_solvency::misc::{generate_random_balances, generate_users};
use kzg_solvency::utils::build_zero_polynomial;
use kzg_solvency::utils::{compute_evaluations_for_specific_omegas, get_omega_domain};
use kzg_solvency::{kzg::KZG, lagrange::lagrange_interpolate};
use ark_std::rand::Rng;

fn open_p_for_user(
    i: usize,
    p_poly: &DensePolynomial<F>,
    kzg: &KZG<Bn254>,
    omegas: &GeneralEvaluationDomain<F>,
    omega_elements: &Vec<F>,
) -> G1 {
    println!(
        "Starting example multi-opening proof generation for user at index {}",
        i
    );
    let started_at = Instant::now();
    let l_evaluations = compute_evaluations_for_specific_omegas::<Bn254>(
        vec![i * 2, i * 2 + 1],
        &omega_elements,
        p_poly,
    );
    let L =
        Evaluations::<F>::from_vec_and_domain(l_evaluations.clone(), omegas.clone()).interpolate();
    let pi = kzg.multi_open(
        p_poly,
        &L,
        vec![omega_elements[i * 2], omega_elements[i * 2 + 1]],
    );
    let duration = started_at.elapsed();
    println!(
        "  (Proved inclusion of (username, balance) at indexes ({}, {}) in {:.2}s))",
        i * 2,
        i * 2 + 1,
        duration.as_secs_f64()
    );

    pi
}

fn verify_p_for_user(
    i: usize,
    omegas: &GeneralEvaluationDomain<F>,
    omega_elements: &Vec<F>,
    l_evaluations: Vec<F>,
    kzg: &KZG<Bn254>,
    p_commitment: G1,
    pi: G1,
) {
    let L = Evaluations::<F>::from_vec_and_domain(l_evaluations, omegas.clone()).interpolate();
    let Z = build_zero_polynomial::<Bn254>(&vec![omega_elements[i * 2], omega_elements[i * 2 + 1]]);

    let verify = kzg.verify_multi_open(p_commitment, pi, &Z, &L);
    assert!(verify);
    println!(
        "Multi opening proof for Constraint 1 verified to {}!",
        verify
    );
}

fn prove_c2(
    n: usize,
    omegas: &GeneralEvaluationDomain<F>,
    kzg: &KZG<Bn254>,
    i_poly: &DensePolynomial<F>,
) -> G1 {
    println!("Starting opening proof for I(ω^(16*x)) = 0 ");
    let start = Instant::now();
    let mut vanishing_omegas: Vec<F> = vec![];

    for i in 0..n {
        vanishing_omegas.push(omegas.element(16 * i));
    }

    // The evaluation of I(X) at the vanishing_omegas should be zero
    let l_evaluations = vec![F::zero(); omegas.elements().count()];

    // The expected opening value for constraint 1 is the evaluation of I(X) at the vanishing_omegas, which should be zero
    let L: DensePolynomial<F> =
        Evaluations::<F>::from_vec_and_domain(l_evaluations.clone(), omegas.clone()).interpolate();

    // Generate opening proof for constraint 1
    let pi = kzg.multi_open(i_poly, &L, vanishing_omegas.clone());
    let duration = start.elapsed();
    println!(
        "  (Proved I(ω^(16*x)) = 0 constraint in {:.2}s))",
        duration.as_secs_f64()
    );

    pi
}

fn verify_c2(
    n: usize,
    omegas: &GeneralEvaluationDomain<F>,
    i_commitment: G1,
    kzg: &KZG<Bn254>,
    pi: G1,
) {
    // Build vanishing polynomial Z(X) in [(P(x) - Q(X)) / Z(X)]
    let mut vanishing_omegas: Vec<F> = vec![];

    for i in 0..n {
        vanishing_omegas.push(omegas.element(16 * i));
    }
    let Z = build_zero_polynomial::<Bn254>(&vanishing_omegas);

    // The evaluation of I(X) at the vanishing_omegas should be zero
    let l_evaluations = vec![F::zero(); omegas.elements().count()];

    // The expected opening value for constraint 1 is the evaluation of I(X) at the vanishing_omegas, which should be zero
    let L: DensePolynomial<F> =
        Evaluations::<F>::from_vec_and_domain(l_evaluations.clone(), omegas.clone()).interpolate();

    // 8. User verifies opening proof for constraint 1 - expect evaluation L(X) = 0
    let verify = kzg.verify_multi_open(i_commitment, pi, &Z, &L);
    println!(
        " Multi opening proof for Constraint 2 verified to {}!",
        verify
    );

    assert!(verify);
}

fn main() {
    // 1. Setup
    let mut rng = test_rng();
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args[1].clone().parse().unwrap();
    println!("Setup with {} random balances and users", n);
    let balances = generate_random_balances(&mut rng, n);
    let users = generate_users(&mut rng, &balances);

    // Sampling a random tau and random generators g1 and g2
    let tau = F::rand(&mut rng);
    let g1 = G1::rand(&mut rng);
    let g2 = G2::rand(&mut rng);

    println!("Generating witness tables");
    let started_at = Instant::now();
    let (p_witness, i_witness) = kzg_solvency::prover::generate_witness::<Bn254>(users).unwrap();
    println!("Generating witness tables took {:?}", started_at.elapsed());

    // 3. Interpolate witness tables into polynomials. i.e. computing P(X) and I(X)
    println!("Computing lagrange interpolation for P(X) and I(X) from witness tables");
    let started_at = Instant::now();
    let p_poly = lagrange_interpolate(&p_witness);
    let i_poly = lagrange_interpolate(&i_witness);
    let poly_degree = p_poly.degree();
    let i_degree = i_poly.degree();

    assert_eq!(poly_degree, i_degree);
    println!(
        "Generating lagrange interpolation took {:?}",
        started_at.elapsed()
    );

    // 4. Initiating KZG and committing to polynomials P(X) and I(X)
    println!("KZG-committing to P(X) and I(X)");
    let started_at = Instant::now();
    let mut kzg_bn254 = KZG::<Bn254>::new(g1, g2, poly_degree);
    kzg_bn254.setup(tau); // setup modifies in place the struct crs
    let p_commitment = kzg_bn254.commit(&p_poly);
    let i_commitment = kzg_bn254.commit(&i_poly);
    println!("KZG-committing took {:?}", started_at.elapsed());

    let k = n * 16;
    let (omegas, omega_elements) = get_omega_domain::<Bn254>(k);

    let mut proof_total_time = Duration::ZERO;
    let mut verify_total_time = Duration::ZERO;

    for i in 0..1 {
        let started = Instant::now();
        let pi = open_p_for_user(i, &p_poly, &kzg_bn254, &omegas, &omega_elements);
        proof_total_time += started.elapsed();

        let l_evaluations = compute_evaluations_for_specific_omegas::<Bn254>(
            vec![i * 2, i * 2 + 1],
            &omega_elements,
            &p_poly,
        );
        let started = Instant::now();
        verify_p_for_user(
            i,
            &omegas,
            &omega_elements,
            l_evaluations,
            &kzg_bn254,
            p_commitment,
            pi,
        );
        verify_total_time += started.elapsed();
    }

    // 7. Generate opening proof for constraint 2: I(ω^(16*x)) = 0.
    let started = Instant::now();
    let pi = prove_c2(n, &omegas, &kzg_bn254, &i_poly);
    proof_total_time += started.elapsed();

    let started = Instant::now();
    verify_c2(n, &omegas, i_commitment, &kzg_bn254, pi);
    verify_total_time += started.elapsed();


    let mut prev_opening: Option<G1> = None;
    for _ in 0..2 {
        // Fiat-Shamir heuristic.
        let x: usize = if let Some(prev_opening) = prev_opening {
            let t = prev_opening.x + prev_opening.y;
            (t.0.0[0] as usize) % n
        } else {
            rng.gen_range(0..n)
        };
        let started = Instant::now();
        let opening = kzg_bn254.open(&i_poly, omega_elements[16 * x + 14], F::from(balances[x]));
        let encrypted_evaluation = kzg_bn254.g1 * F::from(balances[x]);
        proof_total_time += started.elapsed();

        let started = Instant::now();
        let verify = kzg_bn254.verify_from_encrypted_y(
            encrypted_evaluation,
            omega_elements[16 * x + 14],
            i_commitment,
            opening,
        );
        assert!(verify);
        verify_total_time += started.elapsed();

        let started = Instant::now();
        let opening = kzg_bn254.open(&p_poly, omega_elements[2 * x + 1], F::from(balances[x]));
        proof_total_time += started.elapsed();

        let started = Instant::now();
        let verify = kzg_bn254.verify_from_encrypted_y(
            encrypted_evaluation,
            omega_elements[2 * x + 1],
            p_commitment,
            opening,
        );
        assert!(verify);
        verify_total_time += started.elapsed();

        prev_opening = Some(opening);
    }

    let mut prev_opening: Option<G1> = None;
    for _ in 0..2 {
        // Fiat-Shamir heuristic.
        let x: usize = if let Some(prev_opening) = prev_opening {
            let t = prev_opening.x + prev_opening.y;
            ((t.0.0[0] as usize) % n).max(1)
        } else {
            rng.gen_range(1..n)
        };

        let started = Instant::now();
        let z1 = omega_elements[16 * x + 15];
        let y1 = i_poly.evaluate(&z1);
        let opening1 = kzg_bn254.open(&i_poly, z1, y1);
        let encrypted_y1 = kzg_bn254.g1 * y1;

        let z2 = omega_elements[16 * x - 1];
        let y2 = i_poly.evaluate(&z2);
        let opening2 = kzg_bn254.open(&i_poly, z2, y2);
        let encrypted_y2 = kzg_bn254.g1 * y2;

        let z3 = omega_elements[16 * x + 14];
        let y3 = i_poly.evaluate(&z3);
        let opening3 = kzg_bn254.open(&i_poly, z3, y3);
        let encrypted_y3 = kzg_bn254.g1 * y3;
        proof_total_time += started.elapsed();

        let started = Instant::now();
        let verify = kzg_bn254.verify_from_encrypted_y(encrypted_y1, z1, i_commitment, opening1);
        assert!(verify);
        let verify = kzg_bn254.verify_from_encrypted_y(encrypted_y2, z2, i_commitment, opening2);
        assert!(verify);
        let verify = kzg_bn254.verify_from_encrypted_y(encrypted_y3, z3, i_commitment, opening3);
        assert!(verify);
        assert_eq!(encrypted_y1, encrypted_y2 + encrypted_y3);
        verify_total_time += started.elapsed();

        prev_opening = Some(opening1 + opening2 + opening3);
    }


    dbg!(proof_total_time);
    dbg!(verify_total_time);
}

/*
2^10 -- commit 8.13s proof 17.78s verify 1.7s  size 192B
2^11 -- commit 16.7s proof 38.3s verify 4s    size 192B
2^12 -- commit 33.9s proof 80.6s verify 10.8s size 192B
2^13 -- commit 70.1s proof 173.2s verify 22.6s size 192B
 */