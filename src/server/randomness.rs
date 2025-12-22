pub trait RandomGenerator {
    fn next_i32(&mut self) -> i32;
    fn next_i32_range(&mut self, max: u32) -> i32;
    fn next_i64(&mut self) -> i64;
    fn next_f64(&mut self) -> f64;
    fn skip(&mut self, amount: usize);
    fn branch(&mut self) -> Box<dyn RandomGenerator>;
    fn branch_positional(&mut self) -> Box<dyn RandomPositionalGenerator>;
}

pub trait RandomPositionalGenerator: Send + Sync {
    #[inline(always)]
    fn seed_from_pos(x: i32, y: i32, z: i32) -> i64
    where
        Self: Sized,
    {
        let pos_hash = (x as i64 * 3129871) ^ (z as i64 * 116129781) ^ (y as i64);
        ((pos_hash * pos_hash * 42317861) + (pos_hash * 11)) >> 16
    }

    fn pos_to_rand(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomGenerator>;
    fn hash_to_rand(&self, value: &str) -> Box<dyn RandomGenerator>;
}

impl std::fmt::Debug for dyn RandomPositionalGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RandomPositionalGenerator {...}").finish()
    }
}

// Xoroshiro-128bit-plusplus random generator
pub struct Xoroshiro {
    seed_lo: i64,
    seed_hi: i64,
}

pub struct XoroshiroPositional {
    seed_lo: i64,
    seed_hi: i64,
}

impl Xoroshiro {
    pub fn new_from_i64(seed: i64) -> Box<dyn RandomGenerator> {
        let mut seed_128 = Self::to_128bit(seed);
        if (seed_128.0 | seed_128.1) == 0 {
            seed_128.0 = -7046029254386353131;
            seed_128.1 = 7640891576956012809;
        }
        Box::new(Xoroshiro {
            seed_lo: seed_128.0,
            seed_hi: seed_128.1,
        })
    }

    #[inline(always)]
    fn to_128bit(seed: i64) -> (i64, i64) {
        let low = seed ^ 7640891576956012809;
        //let high = low - 7046029254386353131;
        let high = low.wrapping_sub(7046029254386353131);
        (Self::mix_stafford13(low), Self::mix_stafford13(high))
    }

    #[inline(always)]
    fn mix_stafford13(mut seed_part: i64) -> i64 {
        seed_part = (seed_part ^ (u64::from_ne_bytes(seed_part.to_ne_bytes()) >> 30) as i64)
            .wrapping_mul(-4658895280553007687);
        seed_part = (seed_part ^ (u64::from_ne_bytes(seed_part.to_ne_bytes()) >> 27) as i64)
            .wrapping_mul(-7723592293110705685);
        seed_part ^ (u64::from_ne_bytes(seed_part.to_ne_bytes()) >> 31) as i64
    }

    pub fn next_bits(&mut self, bits: i32) -> i64 {
        (u64::from_ne_bytes(self.next_i64().to_ne_bytes()) >> (64 - bits)) as i64
    }
}

impl RandomGenerator for Xoroshiro {
    fn next_i64(&mut self) -> i64 {
        let low = self.seed_lo;
        let mut high = self.seed_hi;
        let new_seed = (low.wrapping_add(high)).rotate_left(17).wrapping_add(low);
        high ^= low;
        self.seed_lo = low.rotate_left(49) ^ high ^ (high << 21);
        self.seed_hi = high.rotate_left(28);
        new_seed
    }

    fn next_i32(&mut self) -> i32 {
        self.next_i64() as i32
    }

    fn next_i32_range(&mut self, max: u32) -> i32 {
        let mut rand_num = self.next_i32() as u32 as u64;
        let mut val = rand_num * max as u64;
        let mut cond = val & 0xFFFFFFFFu64;
        if cond < max as u64 {
            let i = (!max + 1) % max;
            while cond < i as u64 {
                rand_num = self.next_i32() as u32 as u64;
                val = rand_num * max as u64;
                cond = val & 0xFFFFFFFFu64;
            }
        }
        (val >> 32) as i32
    }

    fn next_f64(&mut self) -> f64 {
        self.next_bits(53) as f64 * 1.110223E-16f32 as f64
    }

    fn skip(&mut self, amount: usize) {
        for _ in 0..amount {
            self.next_i64();
        }
    }

    fn branch(&mut self) -> Box<dyn RandomGenerator> {
        Box::new(Xoroshiro {
            seed_lo: self.next_i64(),
            seed_hi: self.next_i64(),
        })
    }

    fn branch_positional(&mut self) -> Box<dyn RandomPositionalGenerator> {
        Box::new(XoroshiroPositional {
            seed_lo: self.next_i64(),
            seed_hi: self.next_i64(),
        })
    }
}

impl RandomPositionalGenerator for XoroshiroPositional {
    fn pos_to_rand(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomGenerator> {
        Box::new(Xoroshiro {
            seed_lo: Self::seed_from_pos(x, y, z) ^ self.seed_lo,
            seed_hi: self.seed_hi,
        })
    }

    fn hash_to_rand(&self, value: &str) -> Box<dyn RandomGenerator> {
        let hash = md5::compute(value).0;
        let mut hash_low = [0u8; 8];
        let mut hash_high = [0u8; 8];
        hash_low.copy_from_slice(&hash[0..8]);
        hash_high.copy_from_slice(&hash[8..16]);
        Box::new(Xoroshiro {
            seed_lo: i64::from_be_bytes(hash_low) ^ self.seed_lo,
            seed_hi: i64::from_be_bytes(hash_high) ^ self.seed_hi,
        })
    }
}

// LCG 48-bit random generator
pub struct LCG48 {
    seed: i64,
}

pub struct LCG48Positional {
    seed: i64,
}

impl LCG48 {
    pub fn new(seed: i64) -> Box<dyn RandomGenerator> {
        Box::new(Self { seed })
    }

    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(25214903917).wrapping_add(11)) & ((1 << 48) - 1);
        self.seed as i32 >> (48 - bits)
    }
}

impl RandomGenerator for LCG48 {
    fn next_i32(&mut self) -> i32 {
        self.next(32)
    }

    fn next_i32_range(&mut self, max: u32) -> i32 {
        if max & (max - 1) == 0 {
            (max as i32 * self.next(31)) >> 31
        } else {
            let mut upper: i32;
            let mut lower: i32;
            while {
                upper = self.next(31);
                lower = upper % max as i32;
                upper - lower + (max as i32 - 1) < 0
            } {}
            lower
        }
    }

    fn next_i64(&mut self) -> i64 {
        ((self.next(32) as i64) << 32) + self.next(32) as i64
    }

    fn next_f64(&mut self) -> f64 {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 * 1.110223E-16f32 as f64
    }

    fn skip(&mut self, amount: usize) {
        for _ in 0..amount {
            self.next_i32();
        }
    }

    fn branch(&mut self) -> Box<dyn RandomGenerator> {
        Box::new(LCG48 {
            seed: self.next_i64(),
        })
    }

    fn branch_positional(&mut self) -> Box<dyn RandomPositionalGenerator> {
        Box::new(LCG48Positional {
            seed: self.next_i64(),
        })
    }
}

impl RandomPositionalGenerator for LCG48Positional {
    fn pos_to_rand(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomGenerator> {
        Box::new(LCG48 {
            seed: Self::seed_from_pos(x, y, z) ^ self.seed,
        })
    }

    fn hash_to_rand(&self, value: &str) -> Box<dyn RandomGenerator> {
        let len = value.len();
        let hash = value.chars().enumerate().fold(0u32, |acc, (i, c)| {
            acc.wrapping_add((c as u32).wrapping_mul(31u32.wrapping_pow((len - (i + 1)) as u32)))
        });
        Box::new(LCG48 {
            seed: hash as i64 ^ self.seed,
        })
    }
}
