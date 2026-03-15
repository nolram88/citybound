pub use rand::{Rng, RngCore};
#[cfg(not(target_arch = "wasm32"))]
pub use rand::thread_rng;
pub use uuid::Uuid;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

// A hashing function with hopefully low correlation between seeds
// but not necessarily good randomness of sequential probes on the same seed
pub struct FnvRng {
    seed: u64,
}

impl RngCore for FnvRng {
    fn next_u64(&mut self) -> u64 {
        let current = self.seed;
        let mut hasher = FnvHasher::default();
        self.seed.hash(&mut hasher);
        self.seed = hasher.finish();
        current
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut offset = 0;
        while offset < bytes.len() {
            let chunk = self.next_u64().to_le_bytes();
            let end = ::std::cmp::min(offset + chunk.len(), bytes.len());
            bytes[offset..end].copy_from_slice(&chunk[..end - offset]);
            offset = end;
        }
    }

    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), ::rand::Error> {
        self.fill_bytes(bytes);
        Ok(())
    }
}

pub fn seed<S: Hash>(seed: S) -> FnvRng {
    let mut hasher = FnvHasher::default();
    seed.hash(&mut hasher);
    FnvRng {
        seed: hasher.finish(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn thread_rng() -> FnvRng {
    use stdweb::unstable::TryInto;

    let seed_value: u32 = js! {
        if (typeof self !== "undefined" && self.crypto && typeof self.crypto.getRandomValues === "function") {
            const data = new Uint32Array(1);
            self.crypto.getRandomValues(data);
            return data[0];
        }

        return Math.floor(Math.random() * 0x100000000);
    }
    .try_into()
    .unwrap_or(0);

    seed(seed_value)
}

pub fn uuid() -> Uuid {
    let mut builder = uuid::Builder::from_bytes(thread_rng().gen());
    builder
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Random)
        .build()
}
