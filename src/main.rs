use std::time::Instant;

use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::{BigInteger, PrimeField, UniformRand};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use ark_std::test_rng;

#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct FGoldilocksConfig;

pub type FGoldilocks = Fp64<MontBackend<FGoldilocksConfig, 1>>;

fn main() {
    let mut rng = test_rng();
    let log_n = 10;
    let log_m = 14;
    let f_coeff = (0..(1 << log_n))
        .map(|_| {
            (0..(1 << log_m))
                .map(|_| <FGoldilocks as UniformRand>::rand(&mut rng))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let domain = Radix2EvaluationDomain::<FGoldilocks>::new(1 << (log_m + 1)).unwrap();

    let start = Instant::now();
    let f_codeword = f_coeff
        .iter()
        .map(|x| domain.fft(x))
        .collect::<Vec<Vec<FGoldilocks>>>();
    println!("{}", start.elapsed().as_millis());

    let a = (0..(1 << log_n) * 8 * 256)
        .map(|_| {
            [0; 8].map(|_| {
                PrimeField::into_bigint(<FGoldilocks as UniformRand>::rand(&mut rng)).0[0] as u128
            })
        })
        .collect::<Vec<_>>();

    let mut hashes = vec![[0u128; 8]; 1 << (log_m + 1)];
    let start = Instant::now();
    for i in 0..(1 << log_n) {
        for j in 0..(1 << (log_m + 1)) {
            let mut cnt = i * 8 * 256;
            let x = PrimeField::into_bigint(f_codeword[i][j]).to_bytes_le();
            assert_eq!(x.len(), 8);
            for k in x {
                let idx = cnt + k as usize;
                hashes[j][0] += a[idx][0];
                hashes[j][1] += a[idx][1];
                hashes[j][2] += a[idx][2];
                hashes[j][3] += a[idx][3];
                hashes[j][4] += a[idx][4];
                hashes[j][5] += a[idx][5];
                hashes[j][6] += a[idx][6];
                hashes[j][7] += a[idx][7];
                cnt += 256;
            }
        }
    }
    for i in hashes.iter_mut() {
        for j in i.iter_mut() {
            *j %= 18446744069414584321;
        }
    }
    println!("{}", start.elapsed().as_millis());

    println!("Shape of A: {} times {}", a[0].len(), a.len() / 256 * 8);
    println!("Shape of F: {} times {}", f_coeff.len(), f_coeff[0].len());
    println!("Shape of B: {} times {}", f_codeword.len() * 64, f_codeword[0].len());
    println!("Shape of C: {} times {}", 8, hashes.len());
}
