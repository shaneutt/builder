// Copyright (c) 2020 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fs::File,
          io::{BufReader,
               Write},
          path::Path,
          str::FromStr};

use crate::hab_core::package::{PackageIdent,
                               PackageTarget};

use habitat_builder_db::models::package::{BuilderPackageIdent,
                                          BuilderPackageTarget,
                                          PackageVisibility,
                                          PackageWithVersionArray};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeType {
    RuntimeDep,
    BuildDep,
    StrongBuildDep,
}

impl Default for EdgeType {
    fn default() -> Self { EdgeType::RuntimeDep }
}

pub fn short_ident(ident: &PackageIdent, use_version: bool) -> PackageIdent {
    let parts: Vec<&str> = ident.iter().collect();
    if use_version {
        PackageIdent::new(parts[0], parts[1], Some(parts[2]), None)
    } else {
        PackageIdent::new(parts[0], parts[1], None, None)
    }
}

pub fn join_idents(sep: &str, identlist: &[PackageIdent]) -> String {
    let strings: Vec<String> = identlist.iter().map(PackageIdent::to_string).collect();
    strings.join(sep)
}

pub fn filter_match(ident: &PackageIdent, filter: Option<&str>) -> bool {
    match filter {
        Some(origin) => ident.origin == origin,
        None => true,
    }
}

pub fn filter_package(package: &PackageWithVersionArray, filter: Option<&str>) -> bool {
    filter_match(&package.ident.0, filter)
}

pub fn write_packages_json<T>(packages: T, filename: &str)
    where T: Iterator<Item = PackageWithVersionArray>
{
    let mut output: Vec<PackageWithVersionArray> = Vec::new();
    for package in packages {
        output.push(package.clone()) // Can I avoid this clone? Maybe need to look at storing things
                                     // as refs/weak refs
    }

    // TODO: figure out how to stream this
    let serialized = serde_json::to_string(&output).unwrap();
    let path = Path::new(filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
}

pub fn read_packages_json(filename: &str) -> Vec<PackageWithVersionArray> {
    let path = Path::new(filename);
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        let u: Vec<PackageWithVersionArray> = serde_json::from_reader(reader).unwrap();
        u
    } else {
        println!("Unable to open file: {:?}", path);
        Vec::new()
    }
}

// Helpers for test

pub fn mk_package_with_versionarray(ident: &str,
                                    target: &str,
                                    rdeps: &[&str],
                                    bdeps: &[&str])
                                    -> PackageWithVersionArray {
    let manifest = format!("\\* __Dependencies__: {}\n\\* __Build Dependencies__: {}\n",
                           rdeps.join(" "),
                           bdeps.join(" "));

    PackageWithVersionArray { ident: BuilderPackageIdent(PackageIdent::from_str(ident).unwrap()),
                              name: ident.to_string(),
                              target:
                                  BuilderPackageTarget(PackageTarget::from_str(target).unwrap()),
                              manifest,
                              deps: mk_builder_package_ident_vec(rdeps),
                              build_deps: mk_builder_package_ident_vec(bdeps),
                              id: 0,
                              owner_id: 0,
                              ident_array: Vec::new(),
                              checksum: String::new(),
                              config: String::new(),
                              tdeps: Vec::new(),
                              exposes: Vec::new(),
                              created_at: None,
                              updated_at: None,
                              visibility: PackageVisibility::Public,
                              origin: String::new(),
                              build_tdeps: Vec::new(),
                              version_array: Vec::new() }
}

pub fn mk_builder_package_ident_vec(vals: &[&str]) -> Vec<BuilderPackageIdent> {
    vals.iter()
        .map(|x| BuilderPackageIdent(PackageIdent::from_str(x).unwrap()))
        .collect()
}

use std::sync::atomic::{AtomicUsize,
                        Ordering};
static TEMP_IDENT_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

pub fn make_temp_ident(ident: &PackageIdent) -> PackageIdent {
    let seq = TEMP_IDENT_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    PackageIdent::new(&ident.origin,
                      &ident.name,
                      ident.version.as_ref(),
                      Some(&format!("N-{}", seq)))
}
