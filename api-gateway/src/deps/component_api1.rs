wit_bindgen::generate!({
    inline: r#"
    package imported:component-api;
    world imports {
        import component:api/request-handler;
    }
    "#,
    with: {
        "component:api/request-handler": generate,
    },
    // Reference the shared WIT package from the repo root
    path: "../wit/world.wit",
    additional_derives: [serde::Deserialize],
    world: "imported:component-api/imports",
});
