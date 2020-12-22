use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use skip_linked_list::skip_linked_list::SkipLinkedList;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

enum Instruction {
    Insert(usize, i32),
    Get(usize),
    Remove(usize),
}

fn simulate_vec(instructions: &Vec<Instruction>) {
    let mut vec = Vec::new();

    for instruction in instructions {
        match instruction {
            Instruction::Insert(i, elem) => vec.insert(*i, elem),
            Instruction::Get(i) => {
                vec[*i];
            },
            Instruction::Remove(i) => {
                vec.remove(*i);
            }
        }
    }
}

fn simulate_skip_linked_list(instructions: &Vec<Instruction>) {
    let mut list = SkipLinkedList::new();

    for instruction in instructions {
        match instruction {
            Instruction::Insert(i, elem) => {
                list.insert(*i, elem);
            },
            Instruction::Get(i) => {
                list.get(*i);
            },
            Instruction::Remove(i) => {
                list.remove(*i);
            },
        }
    }
}

fn generate_instructions(n: usize, weight_insert: usize, weight_get: usize, weight_remove: usize) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    let mut size = 0;
    let weights = vec![weight_insert, weight_get, weight_remove];
    let choices = vec!["i", "g", "r"];
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();
    while instructions.len() < n {
        match choices[dist.sample(&mut rng)] {
            "i" => {
                instructions.push(Instruction::Insert(rng.gen_range(0, size + 1), rng.gen()));
                size += 1;
            },
            "g" if size > 0 => instructions.push(Instruction::Get(rng.gen_range(0, size))),
            "r" if size > 0 => {
                instructions.push(Instruction::Remove(rng.gen_range(0, size)));
                size -= 1;
            },
            _ => (),
        };
    }
    instructions
}

fn bench_writes_heavy(c: &mut Criterion) {
    let sizes = [10000, 50000, 200000];
    let mut group = c.benchmark_group("writes_heavy");
    for n in sizes.iter() {
        let instructions = generate_instructions(*n, 70, 20, 10);
        group.bench_function(BenchmarkId::new("vec ", n), |b| b.iter(|| simulate_vec(&instructions)));
        group.bench_function(BenchmarkId::new("skiplist ", n), |b| b.iter(|| simulate_skip_linked_list(&instructions)));
    }
}

criterion_group!(benches, bench_writes_heavy);
criterion_main!(benches);
