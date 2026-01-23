use std::path::Path;
use syn::{File, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, ItemType};

#[derive(Debug, Clone)]
pub enum CodeItem {
    Struct(StructDef),
    Enum(EnumDef),
    Function(FunctionDef),
    Trait(TraitDef),
    Impl(ImplBlock),
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: String,
    pub fields: Vec<Field>,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub has_phantom_data: bool,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: String,
    pub variants: Vec<Variant>,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub signature: String,
    pub is_method: bool,
    pub self_param: Option<SelfParam>,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelfParam {
    Owned,        // self
    Reference,    // &self
    MutReference, // &mut self
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<String>,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub type_name: String,
    pub methods: Vec<FunctionDef>,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub target: String,
    pub visibility: Visibility,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Restricted,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: String,
    pub line: usize,
}

/// Extract all code items from a parsed syntax tree
pub fn extract_items(file: &File, path: &Path) -> Vec<CodeItem> {
    let mut items = Vec::new();

    for item in &file.items {
        match item {
            Item::Struct(s) => {
                if let Some(code_item) = extract_struct(s, path) {
                    items.push(CodeItem::Struct(code_item));
                }
            }
            Item::Enum(e) => {
                if let Some(code_item) = extract_enum(e, path) {
                    items.push(CodeItem::Enum(code_item));
                }
            }
            Item::Fn(f) => {
                if let Some(code_item) = extract_function(f, path, false) {
                    items.push(CodeItem::Function(code_item));
                }
            }
            Item::Trait(t) => {
                if let Some(code_item) = extract_trait(t, path) {
                    items.push(CodeItem::Trait(code_item));
                }
            }
            Item::Impl(i) => {
                if let Some(code_item) = extract_impl(i, path) {
                    items.push(CodeItem::Impl(code_item));
                }
            }
            Item::Type(ty) => {
                if let Some(code_item) = extract_type_alias(ty, path) {
                    items.push(CodeItem::TypeAlias(code_item));
                }
            }
            _ => {}
        }
    }

    items
}

fn extract_struct(item: &ItemStruct, path: &Path) -> Option<StructDef> {
    let name = item.ident.to_string();
    let item_generics = &item.generics;
    let generics = quote::quote!(#item_generics).to_string();
    let visibility = extract_visibility(&item.vis);
    let doc_comment = extract_doc_comment(&item.attrs);

    let mut fields = Vec::new();
    let mut has_phantom_data = false;

    for field in &item.fields {
        if let Some(field_name) = &field.ident {
            let field_ty = &field.ty;
            let ty_string = quote::quote!(#field_ty).to_string();
            if ty_string.contains("PhantomData") {
                has_phantom_data = true;
            }
            fields.push(Field {
                name: field_name.to_string(),
                ty: ty_string,
            });
        }
    }

    Some(StructDef {
        name,
        generics,
        fields,
        visibility,
        doc_comment,
        has_phantom_data,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_enum(item: &ItemEnum, path: &Path) -> Option<EnumDef> {
    let name = item.ident.to_string();
    let item_generics = &item.generics;
    let generics = quote::quote!(#item_generics).to_string();
    let visibility = extract_visibility(&item.vis);
    let doc_comment = extract_doc_comment(&item.attrs);

    let variants = item
        .variants
        .iter()
        .map(|v| {
            let fields = v
                .fields
                .iter()
                .filter_map(|f| {
                    f.ident.as_ref().map(|name| {
                        let field_ty = &f.ty;
                        Field {
                            name: name.to_string(),
                            ty: quote::quote!(#field_ty).to_string(),
                        }
                    })
                })
                .collect();

            Variant {
                name: v.ident.to_string(),
                fields,
            }
        })
        .collect();

    Some(EnumDef {
        name,
        generics,
        variants,
        visibility,
        doc_comment,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_function(item: &ItemFn, path: &Path, is_method: bool) -> Option<FunctionDef> {
    let name = item.sig.ident.to_string();
    let item_sig = &item.sig;
    let signature = quote::quote!(#item_sig).to_string();
    let visibility = extract_visibility(&item.vis);
    let doc_comment = extract_doc_comment(&item.attrs);

    let self_param = if is_method {
        item.sig.inputs.iter().find_map(|arg| {
            if let syn::FnArg::Receiver(receiver) = arg {
                if receiver.mutability.is_some() {
                    Some(SelfParam::MutReference)
                } else if receiver.reference.is_some() {
                    Some(SelfParam::Reference)
                } else {
                    Some(SelfParam::Owned)
                }
            } else {
                None
            }
        })
    } else {
        None
    };

    Some(FunctionDef {
        name,
        signature,
        is_method,
        self_param,
        visibility,
        doc_comment,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_trait(item: &ItemTrait, path: &Path) -> Option<TraitDef> {
    let name = item.ident.to_string();
    let visibility = extract_visibility(&item.vis);
    let doc_comment = extract_doc_comment(&item.attrs);

    let methods = item
        .items
        .iter()
        .filter_map(|trait_item| {
            if let syn::TraitItem::Fn(method) = trait_item {
                Some(method.sig.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    Some(TraitDef {
        name,
        methods,
        visibility,
        doc_comment,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_impl(item: &ItemImpl, path: &Path) -> Option<ImplBlock> {
    let self_ty = &item.self_ty;
    let type_name = quote::quote!(#self_ty).to_string();
    let trait_name = item.trait_.as_ref().map(|(_, trait_path, _)| {
        quote::quote!(#trait_path).to_string()
    });

    let methods = item
        .items
        .iter()
        .filter_map(|impl_item| {
            if let syn::ImplItem::Fn(method) = impl_item {
                extract_function(
                    &syn::ItemFn {
                        attrs: method.attrs.clone(),
                        vis: method.vis.clone(),
                        sig: method.sig.clone(),
                        block: Box::new(syn::Block {
                            brace_token: Default::default(),
                            stmts: vec![],
                        }),
                    },
                    path,
                    true,
                )
            } else {
                None
            }
        })
        .collect();

    Some(ImplBlock {
        trait_name,
        type_name,
        methods,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_type_alias(item: &ItemType, path: &Path) -> Option<TypeAlias> {
    let name = item.ident.to_string();
    let item_ty = &item.ty;
    let target = quote::quote!(#item_ty).to_string();
    let visibility = extract_visibility(&item.vis);

    Some(TypeAlias {
        name,
        target,
        visibility,
        source_location: SourceLocation {
            file_path: path.to_string_lossy().to_string(),
            line: 1, // Line number tracking would require additional span processing
        },
    })
}

fn extract_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(r) => {
            if r.path.is_ident("crate") {
                Visibility::Crate
            } else {
                Visibility::Restricted
            }
        }
        syn::Visibility::Inherited => Visibility::Private,
    }
}

fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        doc_lines.push(lit_str.value());
                    }
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_struct_with_phantom_data() {
        let code = r#"
use std::marker::PhantomData;

pub struct TypeState<S> {
    data: String,
    _state: PhantomData<S>,
}
"#;
        let file = syn::parse_file(code).unwrap();
        let items = extract_items(&file, Path::new("test.rs"));

        assert_eq!(items.len(), 1);
        match &items[0] {
            CodeItem::Struct(s) => {
                assert_eq!(s.name, "TypeState");
                assert!(s.has_phantom_data);
                assert_eq!(s.visibility, Visibility::Public);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_extract_function_with_self() {
        let code = r#"
impl Foo {
    pub fn consume(self) -> Bar {
        Bar
    }

    pub fn borrow(&self) -> i32 {
        42
    }

    pub fn borrow_mut(&mut self) {
    }
}
"#;
        let file = syn::parse_file(code).unwrap();
        let items = extract_items(&file, Path::new("test.rs"));

        assert_eq!(items.len(), 1);
        match &items[0] {
            CodeItem::Impl(impl_block) => {
                assert_eq!(impl_block.methods.len(), 3);
                assert_eq!(impl_block.methods[0].self_param, Some(SelfParam::Owned));
                assert_eq!(impl_block.methods[1].self_param, Some(SelfParam::Reference));
                assert_eq!(impl_block.methods[2].self_param, Some(SelfParam::MutReference));
            }
            _ => panic!("Expected impl block"),
        }
    }

    #[test]
    fn test_extract_doc_comments() {
        let code = r#"
/// This is a doc comment
/// with multiple lines
pub struct Documented {
    field: i32,
}
"#;
        let file = syn::parse_file(code).unwrap();
        let items = extract_items(&file, Path::new("test.rs"));

        match &items[0] {
            CodeItem::Struct(s) => {
                let doc = s.doc_comment.as_ref().unwrap();
                assert!(doc.contains("This is a doc comment"));
                assert!(doc.contains("with multiple lines"));
            }
            _ => panic!("Expected struct"),
        }
    }
}
