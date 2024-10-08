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

load("@rules_rust//rust:defs.bzl", "rust_library")
load("//bazel:defs.bzl", "either_platform", "select_std_crates")

package(
    default_visibility = ["//visibility:public"],
)

rust_library(
    name = "rust-hypervisor-firmware-virtio",
    srcs = glob(["src/**"]),
    target_compatible_with = either_platform([
        "//:x86_64-none-setting",
        "//:x86_64-none-no_avx-setting",
        "//:x86_64-linux-setting",  # Needed for unit testing of oak_virtio.
    ]),
    deps = [
        "@oak_crates_index//:atomic_refcell",
    ] +
    # TODO: b/347706173 - Try to reduce / remove items in this select by moving all to no_std.
    select_std_crates([
        "bitflags",
        "log",
        "x86_64",
    ]),
)
