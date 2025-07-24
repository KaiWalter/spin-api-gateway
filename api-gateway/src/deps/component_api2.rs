wit_bindgen::generate!({
    inline: r#"
    package imported:component-api;
    world imports {
        import component:api/data-handler;
    }
    "#,
    with: {
        "component:api/data-handler": generate,
    },
    // Reference the shared WIT package from the root directory
    path: "../wit/world.wit",
    additional_derives: [serde::Deserialize],
    world: "imported:component-api/imports",
});
