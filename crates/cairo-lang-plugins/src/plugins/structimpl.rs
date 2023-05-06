use std::sync::Arc;

use cairo_lang_defs::plugin::{
    DynGeneratedFileAuxData, MacroPlugin, PluginGeneratedFile, PluginResult,
};
use cairo_lang_semantic::plugin::{AsDynMacroPlugin, SemanticPlugin, TrivialPluginAuxData};
use cairo_lang_syntax::node::ast::{
    OptionWrappedGenericParamList,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, Terminal, TypedSyntaxNode};
use indoc::formatdoc;
use itertools::Itertools;
use smol_str::SmolStr;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct StructImplPlugin;

impl MacroPlugin for StructImplPlugin {
    fn generate_code(&self, db: &dyn SyntaxGroup, item_ast: ast::Item) -> PluginResult {
        match item_ast {
            ast::Item::AnonymousImpl(impl_ast) => derive_trait(
                db,
                impl_ast
            ),
            _ => PluginResult::default(),
        }
    }
}
impl AsDynMacroPlugin for StructImplPlugin {
    fn as_dyn_macro_plugin<'a>(self: Arc<Self>) -> Arc<dyn MacroPlugin + 'a>
    where
        Self: 'a,
    {
        self
    }
}
impl SemanticPlugin for StructImplPlugin {}

fn derive_trait(
    db: &dyn SyntaxGroup,
    ast: ast::ItemAnonymousImpl,
) -> PluginResult {
    let diagnostics = vec![];
    PluginResult {
        code: Some(PluginGeneratedFile {
            name: "impls".into(),
            content: body(db, &ast),
            aux_data: DynGeneratedFileAuxData(Arc::new(TrivialPluginAuxData {})),
        }),
        diagnostics,
        remove_original_item: true,
    }
}

fn extract_generics(db: &dyn SyntaxGroup, ast: &ast::ItemAnonymousImpl) -> (Vec<SmolStr>, Vec<String>){
    let mut type_generics = vec![];
    let mut other_generics = vec![];
    match ast.generic_params(db) {
        OptionWrappedGenericParamList::WrappedGenericParamList(gens) => gens
            .generic_params(db)
            .elements(db)
            .into_iter()
            .map(|member| match member {
                ast::GenericParam::Type(t) => {
                    type_generics.push(t.name(db).text(db));
                }
                ast::GenericParam::Impl(i) => {
                    other_generics.push(i.as_syntax_node().get_text_without_trivia(db))
                }
                ast::GenericParam::Const(c) => {
                    other_generics.push(c.as_syntax_node().get_text_without_trivia(db))
                }
            })
            .collect(),
        OptionWrappedGenericParamList::Empty(_) => vec![],
    };
    (type_generics, other_generics)
}

fn format_generics(type_generics: &[SmolStr], other_generics: &[String]) -> String {
    format!(
        "<{}{}>",
        type_generics.iter().map(|s| format!("{}, ", s)).collect::<String>(),
        other_generics.iter().map(|s| format!("{}, ", s)).collect::<String>(),
    )
}

fn body(
    db: &dyn SyntaxGroup,
    ast: &ast::ItemAnonymousImpl,
) -> String {
    let name = ast.trait_path(db).as_syntax_node().get_text_without_trivia(db);
    let items = ast.body(db).items(db).elements(db);

    let (type_generics, other_generics) = extract_generics(db, &ast);

    // TODO: warn if there is a conflict with some other thing.
    // TODO: we should check that the functions are indeed methods?
    // TODO: if the user tries to specify a concrete type as a generic arg,
    // the behaviour will be unexpected (the name is treated as a generic arg)
    formatdoc! {"
            trait {name}Methods<{tg}> {{
            {trait}
            }}
            impl {name}Impl{generics} of {name}Methods<{tg}> {body}
        ",
        tg = type_generics.join(", "),
        generics = format_generics(&type_generics, &other_generics),
        body = ast.body(db).as_syntax_node().get_text(db),
        trait = items .iter().map(|item| {
            match item {
                ast::Item::FreeFunction(fn_ast) => {
                    format!("    {};", fn_ast.declaration(db).as_syntax_node().get_text_without_trivia(db))
                }
                // Implementations cannot actually have anything else right now so this should be impossible
                _ => "".into(),
            }
        }).join("\n"),
    }
}