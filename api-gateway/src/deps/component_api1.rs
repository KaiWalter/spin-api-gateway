wit_bindgen::generate!({
    inline: r#"
    package imported:component-api1;
    world imports {
        import component:api1/request-handler;
    }
    "#,
    with: {
        "component:api1/request-handler": generate,
    },
    // Reference the shared WIT package from the repo root
    path: "../wit/world.wit",
    additional_derives: [serde::Deserialize],
    world: "imported:component-api1/imports",
});
