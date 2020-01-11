fn main() {
    let path = std::path::Path::new("src/gl_bindings.rs");
    if !path.exists() {
        let mut file = std::fs::File::create(path).unwrap();
        gl_generator::Registry::new(gl_generator::Api::Gl, (3, 3), gl_generator::Profile::Core, gl_generator::Fallbacks::All, []).write_bindings(gl_generator::StructGenerator, &mut file).unwrap();
    }
}
