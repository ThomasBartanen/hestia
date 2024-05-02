fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}

/*
    Possible styles (all with light and dark variants);
    Fluent
    Material
    Cupertino
    Cosmic

    others:
    qt
    native
*/
