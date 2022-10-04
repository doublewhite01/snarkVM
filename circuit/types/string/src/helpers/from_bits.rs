// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<E: Environment> FromBits for StringType<E> {
    type Boolean = Boolean<E>;

    /// Initializes a new string from a list of little-endian bits *with* trailing zeros (to byte-alignment).
    fn from_bits_le(bits_le: &[Self::Boolean]) -> Self {
        // Determine the mode.
        let mode = Mode::combine(bits_le[0].eject_mode(), bits_le.iter().skip(1).map(|bit| bit.eject_mode()));

        // Inject the string size in bytes.
        let size_in_bytes = Self::inject_size_in_bytes(bits_le, mode);

        // Ensure the list of booleans is byte-aligned.
        let num_bits = bits_le.len();
        match num_bits % 8 == 0 {
            true => StringType {
                mode: bits_le.eject_mode(),
                bytes: bits_le.chunks(8).map(U8::from_bits_le).collect(),
                size_in_bytes,
            },
            false => E::halt(format!("Attempted to instantiate a {num_bits}-bit string, which is not byte-aligned")),
        }
    }

    /// Initializes a new scalar field element from a list of big-endian bits *with* leading zeros (to byte-alignment).
    fn from_bits_be(bits_be: &[Self::Boolean]) -> Self {
        // Determine the mode.
        let mode = Mode::combine(bits_be[0].eject_mode(), bits_be.iter().skip(1).map(|bit| bit.eject_mode()));

        // Inject the string size in bytes.
        let size_in_bytes = Self::inject_size_in_bytes(bits_be, mode);

        // Ensure the list of booleans is byte-aligned.
        let num_bits = bits_be.len();
        match num_bits % 8 == 0 {
            true => StringType {
                mode: bits_be.eject_mode(),
                bytes: bits_be.chunks(8).map(U8::from_bits_be).collect(),
                size_in_bytes,
            },
            false => E::halt(format!("Attempted to instantiate a {num_bits}-bit string, which is not byte-aligned")),
        }
    }
}

impl<E: Environment> StringType<E> {
    /// Checks the size of the given bits for the given mode, and returns the size (of the string) in bytes.
    /// "Load-bearing witness allocation - Please do not optimize me." - Pratyush :)
    fn inject_size_in_bytes(bits: &[Boolean<E>], mode: Mode) -> Field<E> {
        // Ensure the bits are byte-aligned.
        if bits.len() % 8 != 0 {
            E::halt("The given bits for the string are not byte-aligned")
        }

        // Cast the number of bytes in the 'string' as a field element.
        let num_bytes =
            console::Field::from_u32(u32::try_from(bits.len() / 8).unwrap_or_else(|error| E::halt(error.to_string())));

        // Inject the number of bytes as a constant.
        let expected_size_in_bytes = Field::constant(num_bytes);
        // Inject the number of bytes as a witness.
        let size_in_bytes = match mode.is_constant() {
            true => expected_size_in_bytes.clone(),
            false => Field::new(Mode::Private, num_bytes),
        };
        // Ensure the witness matches the constant.
        E::assert_eq(&expected_size_in_bytes, &size_in_bytes);

        // Return the size in bytes.
        size_in_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;

    use rand::Rng;

    const ITERATIONS: u32 = 100;

    fn check_from_bits_le(mode: Mode, num_constants: u64, num_public: u64, num_private: u64, num_constraints: u64) {
        let rng = &mut TestRng::default();

        for i in 0..ITERATIONS {
            // Sample a random string. Take 1/4th to ensure we fit for all code points.
            let expected: String = (0..(Circuit::MAX_STRING_BYTES - i) / 4).map(|_| rng.gen::<char>()).collect();
            let expected_num_bytes = expected.len();
            assert!(expected_num_bytes <= Circuit::MAX_STRING_BYTES as usize);

            let candidate = StringType::<Circuit>::new(mode, console::StringType::new(&expected)).to_bits_le();

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = StringType::<Circuit>::from_bits_le(&candidate);
                assert_eq!(expected, *candidate.eject_value());
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });
            Circuit::reset();
        }
    }

    fn check_from_bits_be(mode: Mode, num_constants: u64, num_public: u64, num_private: u64, num_constraints: u64) {
        let rng = &mut TestRng::default();

        for i in 0..ITERATIONS {
            // Sample a random string. Take 1/4th to ensure we fit for all code points.
            let expected: String = (0..(Circuit::MAX_STRING_BYTES - i) / 4).map(|_| rng.gen::<char>()).collect();
            let expected_num_bytes = expected.len();
            assert!(expected_num_bytes <= Circuit::MAX_STRING_BYTES as usize);

            let candidate = StringType::<Circuit>::new(mode, console::StringType::new(&expected)).to_bits_be();

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = StringType::<Circuit>::from_bits_be(&candidate);
                assert_eq!(expected, *candidate.eject_value());
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });
            Circuit::reset();
        }
    }

    #[test]
    fn test_from_bits_le_constant() {
        check_from_bits_le(Mode::Constant, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_le_public() {
        check_from_bits_le(Mode::Public, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_le_private() {
        check_from_bits_le(Mode::Private, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_be_constant() {
        check_from_bits_be(Mode::Constant, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_be_public() {
        check_from_bits_be(Mode::Public, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_be_private() {
        check_from_bits_be(Mode::Private, 0, 0, 0, 0);
    }
}
