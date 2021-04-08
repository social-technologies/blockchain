// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Traits for dealing with on-chain randomness.

/// A trait that is able to provide randomness.
///
/// Being a deterministic blockchain, real randomness is difficult to come by, different
/// implementations of this trait will provide different security guarantees. At best,
/// this will be randomness which was hard to predict a long time ago, but that has become
/// easy to predict recently.
pub trait Randomness<Output> {
	/// Get a "random" value
	///
	/// Being a deterministic blockchain, real randomness is difficult to come by. This gives you
	/// something that approximates it. At best, this will be randomness which was
	/// hard to predict a long time ago, but that has become easy to predict recently.
	///
	/// `subject` is a context identifier and allows you to get a
	/// different result to other callers of this function; use it like
	/// `random(&b"my context"[..])`.
	fn random(subject: &[u8]) -> Output;

	/// Get the basic random seed.
	///
	/// In general you won't want to use this, but rather `Self::random` which allows you to give a
	/// subject for the random result and whose value will be independently low-influence random
	/// from any other such seeds.
	fn random_seed() -> Output {
		Self::random(&[][..])
	}
}
