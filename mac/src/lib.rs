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

  let mut result = String::from("impl Comp for ");

  result = result + &name;

  result.push_str("{ fn ent_has(ent: EntityRef) -> bool { ent.has::<");
  result = result + &name;
  result.push_str(">() } fn ent_add(world: &mut World, ent: Entity) { world.insert_one(ent, ");
  result = result + &name;
  result.push_str("::default()).expect(\"Could not add component!\"); } ");
  result.push_str("fn ent_rem(world: &mut World, ent: Entity) { world.remove_one::<");
  result = result + &name;
  result.push_str(">(ent); } ");
  /*result.push_str("fn ent_ser<S>(ent: EntityRef, map: S) -> Result<(), S::Error> where S: SerializeMap { ");
  result.push_str("map.serialize_entry(\"");
  result = result + &name;
  result.push_str("\", &*ent.get::<&");
  result = result + &name;
  result.push_str(">().expect(\"Failed to get component!\"))");
  result.push_str(" }");*/
  result.push_str(" }");

  result.parse().unwrap()
}

#[proc_macro]
pub fn define_comps(_: TokenStream) -> TokenStream {
  let mut result = String::from("vec![");
  if let Some(types) = COMP_TYPES.lock().unwrap().as_mut() {
    for ty in types {
      result.push_str("CompType::new::<");
      result = result + ty;
      result.push_str(">(\"");
      result = result + ty;
      result.push_str("\"), ")
    }
  }
  result.push_str("]");
  result.parse().unwrap()
}