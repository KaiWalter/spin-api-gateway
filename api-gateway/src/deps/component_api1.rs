::spin_sdk::wit_bindgen::generate!({
    inline: r#"
    package imported:component-api1;
    world imports {
        import component:api1/data-handler;
    }
    "#,
    with: {
        "component:api1/data-handler": generate,
    },
    path: ".wit/components/deps/api1/component-api1.wit",
});
