use bruteforcer::thread_pool::ThreadPool;
use std::collections::HashSet;

#[test]
fn pool_returns_all_results_with_doubling_fn() {
    let pool: ThreadPool<u32, u32> = ThreadPool::new(4, |x: &u32| x * 2);
    for i in 0u32..100 {
        pool.submit(i).unwrap();
    }
    let results = pool.collect();
    assert_eq!(results.len(), 100);
    let result_set: HashSet<(u32, u32)> = results.into_iter().collect();
    let expected: HashSet<(u32, u32)> = (0u32..100).map(|i| (i, i * 2)).collect();
    assert_eq!(result_set, expected);
}
