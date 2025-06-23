//! Utility functions for cross-platform compatibility

/// Random number generation that works in both client and server contexts
pub mod rand {
    #[cfg(feature = "client")]
    pub fn gen_range<T>(min: T, max: T) -> T
    where
        T: macroquad::rand::RandomRange + PartialOrd + Copy,
    {
        macroquad::rand::gen_range(min, max)
    }

    #[cfg(not(feature = "client"))]
    pub fn gen_range<T>(min: T, max: T) -> T
    where
        T: quad_rand::RandomRange + PartialOrd + Copy,
    {
        quad_rand::gen_range(min, max)
    }
}