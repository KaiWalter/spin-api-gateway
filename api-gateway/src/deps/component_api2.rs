wit_bindgen::generate!({
    inline: r#"
    package imported:component-api2;
    world imports {
        import component:api2/request-handler;
    }
    "#,
    with: {
        "component:api2/request-handler": generate,
    },
    // Reference the shared WIT package from the repo root
    path: "../wit/api2.wit",
    additional_derives: [serde::Deserialize],
    world: "imported:component-api2/imports",
});
