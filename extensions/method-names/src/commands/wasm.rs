use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Wasm {
    path: PathBuf,
}

impl Wasm {
    pub fn process(self) {
        for function in wasmer::Module::from_file(&wasmer::Store::default(), self.path)
            .unwrap()
            .exports()
            .filter(|e| matches!(e.ty(), wasmer::ExternType::Function(_fty)))
        {
            println!("{}", function.name());
        }
    }
}
