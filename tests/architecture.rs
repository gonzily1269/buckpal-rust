use std::collections::BTreeSet;

/// マニフェストからBuckpal内クレートへの依存名を抽出する。
fn project_dependencies(manifest: &str) -> Result<BTreeSet<String>, String> {
    let manifest: toml::Value = toml::from_str(manifest).map_err(|error| error.to_string())?;
    let mut dependencies = BTreeSet::new();
    collect_project_dependencies(&manifest, &mut dependencies);
    Ok(dependencies)
}

/// ワークスペース宣言からメンバーのパスを取り出す。
fn workspace_members(manifest: &str) -> Result<BTreeSet<String>, String> {
    let manifest: toml::Value = toml::from_str(manifest).map_err(|error| error.to_string())?;
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "workspace members must be an array".to_owned())?
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "workspace member names must be strings".to_owned())
        })
        .collect()
}

/// Cargoの各依存セクションを再帰的に走査する。
fn collect_project_dependencies(value: &toml::Value, dependencies: &mut BTreeSet<String>) {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                if matches!(
                    key.as_str(),
                    "dependencies" | "dev-dependencies" | "build-dependencies"
                ) {
                    if let Some(entries) = value.as_table() {
                        for (alias, specification) in entries {
                            let package = specification
                                .get("package")
                                .and_then(toml::Value::as_str)
                                .unwrap_or(alias);
                            if package.starts_with("buckpal-") {
                                dependencies.insert(package.to_owned());
                            }
                        }
                    }
                } else {
                    collect_project_dependencies(value, dependencies);
                }
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_project_dependencies(value, dependencies);
            }
        }
        _ => {}
    }
}

/// 本番依存とbuild依存だけをマニフェストから抽出する。
fn production_dependencies(manifest: &str) -> Result<BTreeSet<String>, String> {
    let manifest: toml::Value = toml::from_str(manifest).map_err(|error| error.to_string())?;
    let mut dependencies = BTreeSet::new();
    collect_production_dependencies(&manifest, &mut dependencies);
    Ok(dependencies)
}

/// Cargoのproduction dependencyセクションを再帰的に走査する。
fn collect_production_dependencies(value: &toml::Value, dependencies: &mut BTreeSet<String>) {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                if matches!(key.as_str(), "dependencies" | "build-dependencies") {
                    if let Some(entries) = value.as_table() {
                        for (alias, specification) in entries {
                            let package = specification
                                .get("package")
                                .and_then(toml::Value::as_str)
                                .unwrap_or(alias);
                            dependencies.insert(package.to_owned());
                        }
                    }
                } else if key != "dev-dependencies" {
                    collect_production_dependencies(value, dependencies);
                }
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_production_dependencies(value, dependencies);
            }
        }
        _ => {}
    }
}

/// プロジェクト内クレート間の依存方向が許可された構造であることを確認する。
#[test]
fn project_crate_dependencies_follow_the_architecture() {
    let cases = [
        (
            ".",
            include_str!("../Cargo.toml"),
            vec![
                "buckpal-adapters".to_owned(),
                "buckpal-application".to_owned(),
                "buckpal-web".to_owned(),
            ],
        ),
        (
            "crates/domain",
            include_str!("../crates/domain/Cargo.toml"),
            Vec::<String>::new(),
        ),
        (
            "crates/application",
            include_str!("../crates/application/Cargo.toml"),
            vec!["buckpal-domain".to_owned()],
        ),
        (
            "crates/adapters",
            include_str!("../crates/adapters/Cargo.toml"),
            vec![
                "buckpal-application".to_owned(),
                "buckpal-domain".to_owned(),
            ],
        ),
        (
            "crates/web",
            include_str!("../crates/web/Cargo.toml"),
            vec!["buckpal-application".to_owned()],
        ),
    ];

    let covered_members = cases
        .iter()
        .map(|(member, _, _)| (*member).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        workspace_members(include_str!("../Cargo.toml")),
        Ok(covered_members)
    );

    for (_, manifest, expected) in cases {
        assert_eq!(
            project_dependencies(manifest),
            Ok(expected.into_iter().collect())
        );
    }
}

/// ドメインとアプリケーションのproduction依存が許可リスト内であることを確認する。
#[test]
fn inner_layers_use_only_approved_production_dependencies() {
    let cases = [
        (
            include_str!("../crates/domain/Cargo.toml"),
            vec!["thiserror".to_owned()],
        ),
        (
            include_str!("../crates/application/Cargo.toml"),
            vec![
                "async-trait".to_owned(),
                "buckpal-domain".to_owned(),
                "thiserror".to_owned(),
            ],
        ),
    ];

    for (manifest, expected) in cases {
        assert_eq!(
            production_dependencies(manifest),
            Ok(expected.into_iter().collect())
        );
    }
}
