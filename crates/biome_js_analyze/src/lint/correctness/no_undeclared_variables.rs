use crate::globals::{is_js_global, is_ts_global};
use crate::services::semantic::SemanticServices;
use biome_analyze::context::RuleContext;
use biome_analyze::{declare_rule, Rule, RuleDiagnostic, RuleSource};
use biome_console::markup;
use biome_js_syntax::{JsFileSource, Language, TextRange, TsAsExpression, TsReferenceType};
use biome_rowan::AstNode;

declare_rule! {
    /// Prevents the usage of variables that haven't been declared inside the document.
    ///
    /// If you need to allow-list some global bindings, you can use the [`javascript.globals`](/reference/configuration/#javascriptglobals) configuration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// foobar;
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// // throw diagnostic for JavaScript files
    /// PromiseLike;
    /// ```
    /// ### Valid
    ///
    /// ```ts
    /// type B<T> = PromiseLike<T>
    /// ```
    pub NoUndeclaredVariables {
        version: "1.0.0",
        name: "noUndeclaredVariables",
        sources: &[RuleSource::Eslint("no-undef")],
        recommended: false,
    }
}

impl Rule for NoUndeclaredVariables {
    type Query = SemanticServices;
    type State = (TextRange, String);
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        ctx.query()
            .all_unresolved_references()
            .filter_map(|reference| {
                let identifier = reference.tree();
                let under_as_expression = identifier
                    .parent::<TsReferenceType>()
                    .and_then(|ty| ty.parent::<TsAsExpression>())
                    .is_some();

                let token = identifier.value_token().ok()?;
                let text = token.text_trimmed();

                let source_type = ctx.source_type::<JsFileSource>();

                if ctx.is_global(text) {
                    return None;
                }

                // Typescript Const Assertion
                if text == "const" && under_as_expression {
                    return None;
                }

                if is_global(text, source_type) {
                    return None;
                }

                let span = token.text_trimmed_range();
                let text = text.to_string();
                Some((span, text))
            })
            .collect()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, (span, name): &Self::State) -> Option<RuleDiagnostic> {
        Some(RuleDiagnostic::new(
            rule_category!(),
            *span,
            markup! {
                "The "<Emphasis>{name}</Emphasis>" variable is undeclared"
            },
        ))
    }
}

fn is_global(reference_name: &str, source_type: &JsFileSource) -> bool {
    match source_type.language() {
        Language::JavaScript => is_js_global(reference_name),
        Language::TypeScript { .. } => is_js_global(reference_name) || is_ts_global(reference_name),
    }
}