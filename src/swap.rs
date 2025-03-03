// Copyright 2021-2022 Farcaster Devs
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA

//! Defines the high level of a swap between a Arbitrating blockchain and a Accordant blockchain
//! and its concrete instances of swaps.

use std::io;
use std::str::FromStr;

use crate::consensus::{self, Decodable, Encodable};
use crate::hash::HashString;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub mod btcxmr;

fixed_hash::construct_fixed_hash!(
    /// A unique swap identifier represented as an 32 bytes hash.
    pub struct SwapId(32);
);

impl Serialize for SwapId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(format!("{:#x}", self).as_ref())
    }
}

impl<'de> Deserialize<'de> for SwapId {
    fn deserialize<D>(deserializer: D) -> Result<SwapId, D::Error>
    where
        D: Deserializer<'de>,
    {
        SwapId::from_str(&deserializer.deserialize_string(HashString)?).map_err(de::Error::custom)
    }
}

impl Encodable for SwapId {
    fn consensus_encode<W: io::Write>(&self, s: &mut W) -> Result<usize, io::Error> {
        self.0.consensus_encode(s)
    }
}

impl Decodable for SwapId {
    fn consensus_decode<D: io::Read>(d: &mut D) -> Result<Self, consensus::Error> {
        let bytes: [u8; 32] = Decodable::consensus_decode(d)?;
        Ok(Self::from_slice(&bytes))
    }
}

impl_strict_encoding!(SwapId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_swapid_in_yaml() {
        let swap_id =
            SwapId::from_str("0x1baf1b36075de25a0f8e914b36759cac6f5d825622f8ccee597d87d4850c0d38")
                .expect("Valid hex string");
        let s = serde_yaml::to_string(&swap_id).expect("Encode swap id in yaml");
        assert_eq!(
            "---\n\"0x1baf1b36075de25a0f8e914b36759cac6f5d825622f8ccee597d87d4850c0d38\"\n",
            s
        );
    }

    #[test]
    fn deserialize_swapid_from_yaml() {
        let s = "---\n\"0x1baf1b36075de25a0f8e914b36759cac6f5d825622f8ccee597d87d4850c0d38\"\n";
        let swap_id = serde_yaml::from_str(&s).expect("Decode swap id from yaml");
        assert_eq!(
            SwapId::from_str("0x1baf1b36075de25a0f8e914b36759cac6f5d825622f8ccee597d87d4850c0d38")
                .expect("Valid hex string"),
            swap_id
        );
    }
}
