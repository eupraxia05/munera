extern crate proc_macro;
use proc_macro::TokenStream;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::default;
use syn::{ parse_macro_input, DeriveInput, DataStruct };
use syn::Data::Struct;

type CompTypeVec = Vec<String>;

lazy_static! {
  static ref COMP_TYPES: Mutex<Option<CompTypeVec>> = Mutex::new(Some(CompTypeVec::new()));
}

#[proc_macro_derive(Comp)]
pub fn comp(item: TokenStream) -> TokenStream
{
  let DeriveInput { ident, data, .. } = parse_macro_input!(item as DeriveInput);

  let name = ident.to_string();

  if let Some(types) = COMP_TYPES.lock().unwrap().as_mut() {
    types.push(name.clone());
  }

  let mut result = String::from("#[typetag::serde]");
  result.push_str("#[allow(non_snake_case)]");
  result.push_str("impl crate::engine::Comp for ");

  result = result + &name;

  result.push_str("{");
  result.push_str("fn as_any(&self) -> &dyn std::any::Any { self }");
  result.push_str("}");
  result.push_str(format!("impl crate::engine::CompExt for {} {{", name).as_str());
  result.push_str("}");
  result.push_str(format!("inventory::submit! {{ crate::engine::CompType::new::<{}>(\"{}\") }}", name, name).as_str());

  result.parse().unwrap()
}

#[proc_macro_derive(Asset)]
pub fn asset(item: TokenStream) -> TokenStream {
  let DeriveInput {ident, data, ..} = parse_macro_input!(item as DeriveInput);

  let name = ident.to_string();

  let mut result = String::new();
  result.push_str(format!("impl munera_assets::Asset for {} {{", name).as_str());
  result.push_str("fn as_any(&self) -> &dyn std::any::Any { self }");
  result.push_str("fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }");
  result.push_str("}");
  result.push_str(format!("impl munera_assets::AssetExt for {} {{", name).as_str());
  result.push_str(format!("fn asset_type_name() -> &'static str {{ \"{}\" }}", name).as_str());
  result.push_str("}");
  result.push_str(format!("inventory::submit! {{ munera_assets::AssetType::new::<{}>() }}", name).as_str());
  result.parse().unwrap()
}