#
# Copyright 2024 The Project Oak Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
#
"""
 Defines the no_std crates repository.
 After being defined, it should be loaded using:

 load("@oak_crates_index//:defs.bzl", oak_no_std_crate_repositories = "crate_repositories")

 oak_no_std_crate_repositories()
 """

load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository")
load("//bazel/rust:defs.bzl", "RUST_NIGHTLY_VERSION")

def oak_no_std_crates_index(cargo_lockfile, lockfile, index_name = "oak_no_std_crates_index", extra_annotations = {}, extra_packages = {}):
    # All creates in this repository must support no_std.
    crates_repository(
        name = index_name,
        annotations = extra_annotations,
        cargo_lockfile = cargo_lockfile,  # In Cargo-free mode this is used as output, not input.
        lockfile = lockfile,  # Shares most contents with cargo_lockfile.
        packages = {
            "acpi": crate.spec(version = "*"),
            "aead": crate.spec(version = "*"),
            "aes-gcm": crate.spec(
                default_features = False,
                features = [
                    "aes",
                    "alloc",
                ],
                version = "*",
            ),
            "aml": crate.spec(version = "*"),
            "anyhow": crate.spec(
                default_features = False,
                features = [],
                version = "*",
            ),
            "bytes": crate.spec(
                default_features = False,  # bytes crate has "std" in its default feature set.
                version = "*",
            ),
            "bitflags": crate.spec(
                package = "bitflags",
                version = "*",
            ),
            "ciborium": crate.spec(
                default_features = False,
                version = "*",
            ),
            "coset": crate.spec(
                default_features = False,
                version = "*",
            ),
            # Pin to 4.1.1 see issue #4952
            # TODO: #4952 - Remove this pinning.
            "curve25519-dalek": crate.spec(
                default_features = False,
                version = "=4.1.1",
            ),
            "ecdsa": crate.spec(
                default_features = False,
                features = [
                    "der",
                    "pem",
                    "pkcs8",
                    "signing",
                ],
                version = "*",
            ),
            "elf": crate.spec(
                default_features = False,
                version = "*",
            ),
            "getrandom": crate.spec(
                default_features = False,
                # rdrand is required to support x64_64-unknown-none.
                features = ["rdrand"],
                version = "0.2.12",
            ),
            "goblin": crate.spec(
                default_features = False,
                features = [
                    "elf32",
                    "elf64",
                    "endian_fd",
                ],
                version = "*",
            ),
            "hashbrown": crate.spec(
                default_features = False,
                version = "0.14",
            ),
            "hex": crate.spec(
                default_features = False,
                features = ["alloc"],
                version = "*",
            ),
            "hkdf": crate.spec(
                default_features = False,
                version = "*",
            ),
            "hpke": crate.spec(
                default_features = False,
                features = [
                    "alloc",
                    "x25519",
                ],
                version = "*",
            ),
            "lazy_static": crate.spec(
                features = ["spin_no_std"],
                version = "*",
            ),
            "lock_api": crate.spec(
                default_features = False,
                features = [],
                version = "*",
            ),
            "log": crate.spec(
                features = [],
                version = "*",
            ),
            "maplit": crate.spec(
                features = [],
                version = "*",
            ),
            "p256": crate.spec(
                default_features = False,
                features = [
                    "alloc",
                    "ecdsa",
                    "pem",
                ],
                version = "*",
            ),
            "pkcs8": crate.spec(
                default_features = False,
                features = ["alloc"],
                version = "*",
            ),
            "primeorder": crate.spec(
                default_features = False,
                version = "*",
            ),
            "prost": crate.spec(
                default_features = False,
                # No derive feature - it requires std and will make other crates
                # in this index, like bytes, require std.
                version = "*",
            ),
            "rand_core": crate.spec(
                default_features = False,
                features = ["getrandom"],
                version = "*",
            ),
            "sha2": crate.spec(
                default_features = False,
                version = "*",
            ),
            "spinning_top": crate.spec(version = "*"),
            "static_assertions": crate.spec(version = "*"),
            "strum": crate.spec(
                default_features = False,
                features = ["derive"],
                version = "*",
            ),
            "virtio-drivers": crate.spec(version = "*"),
            "x86_64": crate.spec(version = "=0.14"),  #  0.15 does not support LowerHex formatting.
            "zeroize": crate.spec(
                features = ["derive"],
                version = "*",
            ),
            "rand_chacha": crate.spec(
                default_features = False,
                version = "*",
            ),
            "wasmi": crate.spec(
                default_features = False,
                # same version as cargo, newer versions had compatibility issues
                version = "0.31.2",
            ),
        } | extra_packages,
        rust_version = RUST_NIGHTLY_VERSION,
        supported_platform_triples = [
            # Linux for dependencies of build scripts (they run on host):
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-none",
            "wasm32-unknown-unknown",
        ],
    )
