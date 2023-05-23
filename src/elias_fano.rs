use crate::bit_vec::{BitVec, BuildingStrategy};
use crate::RsVector;

pub struct EliasFanoVec<B: RsVector> {
    upper_vec: B,
    lower_vec: BitVec,
    lower_len: usize,
}

impl<B: RsVector + BuildingStrategy<Vector = B>> EliasFanoVec<B> {
    /// Create a new Elias-Fano vector by compressing the given data.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(data: &Vec<u64>) -> Self {
        let lower_width = data.len().leading_zeros() as usize + 1;

        let mut upper_vec = BitVec::from_zeros(data.len() * 2 + 1);
        let mut lower_vec = BitVec::with_capacity(data.len() * lower_width);

        for (i, &word) in data.iter().enumerate() {
            let upper = (word >> lower_width) as usize;
            let lower = word & ((1 << lower_width) - 1);

            upper_vec.flip_bit(upper + i + 1);
            lower_vec.append_bits(lower, lower_width);
        }

        Self {
            upper_vec: B::from_bit_vec(upper_vec),
            lower_vec,
            lower_len: lower_width,
        }
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.lower_vec.len() / self.lower_len
    }

    /// Returns true if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the element at the given index.
    #[allow(clippy::cast_possible_truncation)]
    pub fn get(&self, index: usize) -> u64 {
        let upper = self.upper_vec.select1(index) - index - 1;
        let lower = self
            .lower_vec
            .get_bits(index * self.lower_len, self.lower_len);
        (upper << self.lower_len) as u64 | lower
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn pred(&self, n: u64) -> u64 {
        let upper = n >> self.lower_len;
        let lower_bound = self.upper_vec.rank1(self.upper_vec.select0(upper as usize));
        let upper_bound = self
            .upper_vec
            .rank1(self.upper_vec.select0(upper as usize + 1));

        let mut lower_candidate = self
            .lower_vec
            .get_bits(lower_bound * self.lower_len, self.lower_len);
        let mut result_upper =
            ((self.upper_vec.select1(lower_bound) - lower_bound - 1) << self.lower_len) as u64;

        if lower_bound < upper_bound && (result_upper | lower_candidate) <= n {
            for i in ((lower_bound + 1) * self.lower_len..upper_bound * self.lower_len)
                .step_by(self.lower_len)
            {
                let next_candidate = self.lower_vec.get_bits(i, self.lower_len);
                if result_upper | next_candidate > n {
                    return result_upper | lower_candidate;
                } else {
                    lower_candidate = next_candidate;
                }
            }
        } else {
            result_upper =
                ((self.upper_vec.select1(lower_bound - 1) - lower_bound) << self.lower_len) as u64;
            lower_candidate = self
                .lower_vec
                .get_bits(lower_bound * self.lower_len - 1, self.lower_len);
        }

        result_upper | lower_candidate
    }
}

#[cfg(test)]
mod tests {
    use crate::{EliasFanoVec, FastBitVector};
    use rand::{thread_rng, Rng};

    #[test]
    fn test_elias_fano() {
        let ef = EliasFanoVec::<FastBitVector>::new(&vec![0, 1, 4, 7]);

        assert_eq!(ef.len(), 4);
        assert_eq!(ef.get(0), 0);
        assert_eq!(ef.get(1), 1);
        assert_eq!(ef.get(2), 4);
        assert_eq!(ef.get(3), 7);

        assert_eq!(ef.pred(0), 0);
        assert_eq!(ef.pred(1), 1);
        assert_eq!(ef.pred(2), 1);
        assert_eq!(ef.pred(5), 4);
        assert_eq!(ef.pred(8), 7);
    }

    #[test]
    fn test_randomized_elias_fano() {
        let mut rng = thread_rng();
        let mut seq = vec![0u64; 1000];
        for i in 0..1000 {
            seq[i] = rng.gen();
        }
        seq.sort_unstable();

        let ef = EliasFanoVec::<FastBitVector>::new(&seq);

        assert_eq!(ef.len(), seq.len());

        for (i, &v) in seq.iter().enumerate() {
            assert_eq!(ef.get(i), v);
        }

        for _ in 0..1000 {
            let random_splitter: u64 = rng.gen();
            let partition_point = seq.partition_point(|&x| x <= random_splitter);
            let pred = ef.pred(random_splitter);

            assert_eq!(
                ef.pred(random_splitter),
                seq[seq.partition_point(|&x| x <= random_splitter) - 1]
            );
        }
    }
}
