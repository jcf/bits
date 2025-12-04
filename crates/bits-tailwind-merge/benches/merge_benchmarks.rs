use bits_tailwind_merge::tw_merge;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_simple_merge(c: &mut Criterion) {
    c.bench_function("simple merge (no conflicts)", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "flex items-center justify-center bg-red-500 text-white p-4",
            ))
        })
    });
}

fn bench_merge_with_conflicts(c: &mut Criterion) {
    c.bench_function("merge with conflicts", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "px-2 py-1 p-4 bg-red-500 bg-blue-500 text-sm text-lg hover:p-2 hover:p-4",
            ))
        })
    });
}

fn bench_many_duplicates(c: &mut Criterion) {
    let input = vec!["p-4"; 100].join(" ");
    c.bench_function("pathological: 100 duplicates", |b| {
        b.iter(|| tw_merge(black_box(&input)))
    });
}

fn bench_complex_real_world(c: &mut Criterion) {
    c.bench_function("real-world complex", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "flex items-center justify-between p-4 hover:bg-gray-100 \
                 dark:bg-gray-800 dark:hover:bg-gray-700 sm:p-6 md:p-8 \
                 border border-gray-200 rounded-lg shadow-sm hover:shadow-md \
                 transition-all duration-200 gap-4 w-full max-w-2xl mx-auto \
                 text-gray-900 dark:text-gray-100 font-medium text-base \
                 bg-white p-6",
            ))
        })
    });
}

fn bench_deep_modifiers(c: &mut Criterion) {
    c.bench_function("deep modifiers", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "hover:focus:active:dark:sm:md:lg:xl:p-4 \
                 hover:focus:active:dark:sm:md:lg:xl:p-2",
            ))
        })
    });
}

fn bench_arbitrary_values(c: &mut Criterion) {
    c.bench_function("arbitrary values", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "bg-[#B91C1C] bg-[rgb(255,0,0)] w-[500px] w-[calc(100%-20px)] \
                 p-[1.5rem] p-[20px]",
            ))
        })
    });
}

fn bench_many_classes_no_conflicts(c: &mut Criterion) {
    c.bench_function("50 non-conflicting classes", |b| {
        b.iter(|| {
            tw_merge(black_box(
                "flex items-center justify-center bg-red-500 text-white p-4 m-2 \
                 border rounded shadow hover:bg-red-600 focus:ring dark:bg-red-700 \
                 sm:text-lg md:text-xl lg:text-2xl gap-2 w-full h-full opacity-90 \
                 transition duration-200 ease-in-out transform scale-100 rotate-0 \
                 font-bold text-center align-middle z-10 cursor-pointer select-none \
                 overflow-hidden whitespace-nowrap text-ellipsis block relative \
                 top-0 left-0 right-0 bottom-0 inset-0 visible appearance-none",
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_simple_merge,
    bench_merge_with_conflicts,
    bench_many_duplicates,
    bench_complex_real_world,
    bench_deep_modifiers,
    bench_arbitrary_values,
    bench_many_classes_no_conflicts,
);
criterion_main!(benches);
