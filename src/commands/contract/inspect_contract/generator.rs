use near_abi::{AbiFunctionKind, AbiParameters, AbiRoot, AbiType};
use near_schemafy_lib::{Expander, Generator, Schema};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::path::{Path, PathBuf};

pub fn generate_abi_client(
    near_abi: AbiRoot,
    contract_name: proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let schema_json = serde_json::to_string(&near_abi.body.root_schema).unwrap();

    let generator = Generator::builder().with_input_json(&schema_json).build();
    let (mut token_stream, schema) = generator.generate_with_schema();
    let mut expander = Expander::new(None, "", &schema);

    token_stream.extend(quote! {
        pub struct #contract_name {
            pub contract: near_workspaces::Contract,
        }
    });

    let mut methods_stream = proc_macro2::TokenStream::new();

    for function in near_abi.body.functions {
        let name = format_ident!("{}", function.name);

        let mut param_names = vec![];
        let params = match &function.params {
            AbiParameters::Borsh { .. } => panic!("Borsh is currently unsupported"),
            AbiParameters::Json { args } => args
                .iter()
                .map(|arg| {
                    param_names.push(format_ident!("{}", arg.name));
                    let arg_name = param_names.last().unwrap();
                    let arg_type = expand_subschema(&mut expander, &arg.type_schema);
                    quote! { #arg_name: #arg_type }
                })
                .collect::<Vec<_>>(),
        };

        let return_type = function.result;
        // .map(|r_type| match r_type {
        //     AbiType::Json { type_schema } => {
        //         expand_subschema(&mut expander, &type_schema)
        //     },
        //     AbiType::Borsh { type_schema: _ } => panic!("Borsh is currently unsupported"),
        // });
        // .unwrap_or_else(|| quote!());
        // .unwrap_or_else(|| format_ident!("{}", "()"));
        let return_type = match return_type {
            Some(r_type) => {
                let ty = match r_type {
                    AbiType::Json { type_schema } => expand_subschema(&mut expander, &type_schema),
                    AbiType::Borsh { type_schema: _ } => panic!("Borsh is currently unsupported"),
                };

                quote! { #ty }
            }
            None => quote!(),
        };

        let name_str = name.to_string();
        let args = if param_names.is_empty() {
            // Special case for parameter-less functions because otherwise the type for
            // `[]` is not inferrable.
            quote! { () }
        } else {
            quote! { [#(#param_names),*] }
        };
        if function.kind == AbiFunctionKind::View {
            methods_stream.extend(quote! {
                    pub async fn #name(
                        &self,
                        #(#params),*
                    ) -> anyhow::Result<#return_type> {
                        let result = self.contract
                        .call(#name_str)
                        .args_json(#args)
                        .view()
                        .await?;
                    Ok(result.json::<#return_type>()?)
                }
            });
        } else {
            methods_stream.extend(quote! {
            pub async fn #name(
                &self,
                gas: near_workspaces::types::Gas,
                        deposit: near_workspaces::types::Balance,
                        #(#params),*
                    ) -> anyhow::Result<#return_type> {
                        let result = self.contract
                            .call(#name_str)
                            .args_json(#args)
                            .gas(gas)
                            .deposit(deposit)
                            .transact()
                            .await?;
                        Ok(result.json::<#return_type>()?)
                    }
                });
        }
    }

    token_stream.extend(quote! {
        impl #contract_name {
            #methods_stream
        }
    });

    token_stream
}

pub fn read_abi(abi_path: impl AsRef<Path>) -> AbiRoot {
    let abi_path = if abi_path.as_ref().is_relative() {
        let crate_root = get_crate_root().unwrap();
        crate_root.join(&abi_path)
    } else {
        PathBuf::from(abi_path.as_ref())
    };

    let abi_json = std::fs::read_to_string(&abi_path)
        .unwrap_or_else(|err| panic!("Unable to read `{}`: {}", abi_path.to_string_lossy(), err));

    serde_json::from_str::<AbiRoot>(&abi_json).unwrap_or_else(|err| {
        panic!(
            "Cannot parse `{}` as ABI: {}",
            abi_path.to_string_lossy(),
            err
        )
    })
}

fn get_crate_root() -> std::io::Result<PathBuf> {
    if let Ok(path) = std::env::var("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(path));
    }

    let current_dir = std::env::current_dir()?;

    for p in current_dir.ancestors() {
        if std::fs::read_dir(p)?
            .filter_map(Result::ok)
            .any(|p| p.file_name().eq("Cargo.toml"))
        {
            return Ok(PathBuf::from(p));
        }
    }

    Ok(current_dir)
}

fn schemars_schema_to_schemafy(schema: &schemars::schema::Schema) -> Schema {
    let schema_json = serde_json::to_string(&schema).unwrap();
    serde_json::from_str(&schema_json).unwrap_or_else(|err| {
        panic!(
            "Could not convert schemars schema to schemafy model: {}",
            err
        )
    })
}

fn expand_subschema(
    expander: &mut Expander,
    schema: &schemars::schema::Schema,
) -> proc_macro2::Ident {
    let schemafy_schema = schemars_schema_to_schemafy(schema);
    format_ident!("{}", expander.expand_type_from_schema(&schemafy_schema).typ)
}
