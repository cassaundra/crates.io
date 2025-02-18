use crate::builders::{CrateBuilder, VersionBuilder};
use crate::util::insta::{self, assert_yaml_snapshot};
use crate::util::{RequestHelper, TestApp};
use diesel::prelude::*;
use serde_json::Value;

#[test]
fn show_by_crate_name_and_version() {
    let (app, anon, user) = TestApp::init().with_user();
    let user = user.as_model();

    let v = app.db(|conn| {
        let krate = CrateBuilder::new("foo_vers_show", user.id).expect_build(conn);
        VersionBuilder::new("2.0.0")
            .size(1234)
            .checksum("c241cd77c3723ccf1aa453f169ee60c0a888344da504bee0142adb859092acb4")
            .expect_build(krate.id, user.id, conn)
    });

    let url = "/api/v1/crates/foo_vers_show/2.0.0";
    let json: Value = anon.get(url).good();
    assert_yaml_snapshot!(json, {
        ".version.id" => insta::id_redaction(v.id),
        ".version.created_at" => "[datetime]",
        ".version.updated_at" => "[datetime]",
        ".version.published_by.id" => insta::id_redaction(user.id),
    });
}

#[test]
fn show_by_crate_name_and_semver_no_published_by() {
    use cargo_registry::schema::versions;
    use diesel::{update, RunQueryDsl};

    let (app, anon, user) = TestApp::init().with_user();
    let user = user.as_model();

    let v = app.db(|conn| {
        let krate = CrateBuilder::new("foo_vers_show_no_pb", user.id).expect_build(conn);
        let version = VersionBuilder::new("1.0.0").expect_build(krate.id, user.id, conn);

        // Mimic a version published before we started recording who published versions
        let none: Option<i32> = None;
        update(versions::table)
            .set(versions::published_by.eq(none))
            .execute(conn)
            .unwrap();

        version
    });

    let url = "/api/v1/crates/foo_vers_show_no_pb/1.0.0";
    let json: Value = anon.get(url).good();
    assert_yaml_snapshot!(json, {
        ".version.id" => insta::id_redaction(v.id),
        ".version.created_at" => "[datetime]",
        ".version.updated_at" => "[datetime]",
    });
}
