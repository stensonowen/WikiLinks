
// simpler bloom filter that should allow us to cheapen a super common op

type UNIT = u64;
const UNIT_SIZE: usize = 64; // eventually ::std::mem::size_of::<UNIT>()

//const SIZE: usize = 1 << 10; // 1024 bytes (fair for the stack)
const SIZE: usize = 1 << 7;
const ARR_LEN: usize = SIZE / UNIT_SIZE; // div by bits of 


pub struct Bloom([UNIT; ARR_LEN]);
//pub struct Bloom(Box<[UNIT; ARR_LEN]>);

impl Bloom {
    pub fn new() -> Bloom {
        Bloom([0; ARR_LEN])
        //Bloom(Box::new([0; ARR_LEN]))
    }
    fn _get(&self, i: usize) -> bool {
        // These ops optimize to shifts when built with -O
        let (q, r) = (i / UNIT_SIZE, i % UNIT_SIZE);
        (self.0[q] & (1 << r)) != 0
    }
    fn _set(&mut self, i: usize) {
        let (q, r) = (i / UNIT_SIZE, i % UNIT_SIZE);
        self.0[q] |= 1 << r;
    }
    pub fn get_and_set(&mut self, i: u32) -> bool {
        let i = i as usize;
        let (q, r) = ((i / UNIT_SIZE) % ARR_LEN, i % UNIT_SIZE);
        let ret = (self.0[q] & (1 << r)) != 0;
        self.0[q] |= 1 << r;
        ret
    }
}

/*

 On Stack w/ SIZE=1<<10

    running 6 tests
    test tests::bfs_long     ... bench:     751,183 ns/iter (+/- 149,080)
    test tests::bfs_long_2   ... bench:     703,833 ns/iter (+/- 127,376)
    test tests::bfs_medium   ... bench:     321,910 ns/iter (+/- 19,067)
    test tests::bfs_medium_2 ... bench:     327,730 ns/iter (+/- 29,030)
    test tests::bfs_small    ... bench:     182,646 ns/iter (+/- 17,628)
    test tests::bfs_small_2  ... bench:     446,567 ns/iter (+/- 47,500)

    running 6 tests
    test tests::bfs_long     ... bench:     746,474 ns/iter (+/- 58,955)
    test tests::bfs_long_2   ... bench:     728,273 ns/iter (+/- 84,210)
    test tests::bfs_medium   ... bench:     322,174 ns/iter (+/- 41,901)
    test tests::bfs_medium_2 ... bench:     331,449 ns/iter (+/- 30,692)
    test tests::bfs_small    ... bench:     182,467 ns/iter (+/- 18,858)
    test tests::bfs_small_2  ... bench:     446,358 ns/iter (+/- 53,492)

 On Heap w/ SIZE=1<<10

    running 6 tests
    test tests::bfs_long     ... bench:     731,025 ns/iter (+/- 95,132)
    test tests::bfs_long_2   ... bench:     710,335 ns/iter (+/- 58,055)
    test tests::bfs_medium   ... bench:     316,470 ns/iter (+/- 26,222)
    test tests::bfs_medium_2 ... bench:     329,662 ns/iter (+/- 20,987)
    test tests::bfs_small    ... bench:     179,947 ns/iter (+/- 32,596)
    test tests::bfs_small_2  ... bench:     438,112 ns/iter (+/- 31,333)

 On Heap w/ SIZE=1<<10



    */
